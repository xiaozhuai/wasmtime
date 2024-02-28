use super::{RunCommand, RunTarget};
use anyhow::{anyhow, bail, Context, Result};
use std::sync::Arc;
use wasi_common::sync::{TcpListener, WasiCtxBuilder};
use wasmtime::{Store, StoreLimits};

#[cfg(feature = "wasi-nn")]
use wasmtime_wasi_nn::WasiNnCtx;
#[cfg(feature = "wasi-threads")]
use wasmtime_wasi_threads::WasiThreadsCtx;

#[derive(Clone)]
pub struct Host {
    wasi: wasi_common::WasiCtx,
    #[cfg(feature = "wasi-threads")]
    wasi_threads: Option<Arc<WasiThreadsCtx<Host>>>,
    #[cfg(feature = "wasi-nn")]
    wasi_nn: Option<Arc<WasiNnCtx>>,
    limits: StoreLimits,
    #[cfg(feature = "profiling")]
    guest_profiler: Option<Arc<wasmtime::GuestProfiler>>,
}

impl RunCommand {
    fn build_wasi_ctx(&self) -> Result<wasi_common::WasiCtx> {
        let mut builder = WasiCtxBuilder::new();
        builder.inherit_stdio().args(&self.compute_argv()?)?;

        for (key, value) in self.vars.iter() {
            let value = match value {
                Some(value) => value.clone(),
                None => std::env::var(key)
                    .map_err(|_| anyhow!("environment variable `{key}` not found"))?,
            };
            builder.env(key, &value)?;
        }

        let mut num_fd: usize = 3;

        if self.run.common.wasi.listenfd == Some(true) {
            num_fd = ctx_set_listenfd(num_fd, &mut builder)?;
        }

        for listener in self.compute_preopen_sockets()? {
            builder.preopened_socket(num_fd as _, listener)?;
            num_fd += 1;
        }

        for (name, dir) in self.compute_preopen_dirs()? {
            builder.preopened_dir(dir, name)?;
        }

        Ok(builder.build())
    }

    fn compute_preopen_sockets(&self) -> Result<Vec<TcpListener>> {
        let mut listeners = vec![];

        for address in &self.run.common.wasi.tcplisten {
            let stdlistener = std::net::TcpListener::bind(address)
                .with_context(|| format!("failed to bind to address '{}'", address))?;

            let _ = stdlistener.set_nonblocking(true)?;

            listeners.push(TcpListener::from_std(stdlistener))
        }
        Ok(listeners)
    }

    pub(super) fn execute_legacy(
        mut self,
        engine: wasmtime::Engine,
        main: wasmtime::Module,
    ) -> Result<()> {
        let mut linker: wasmtime::Linker<Host> = wasmtime::Linker::new(&engine);
        if let Some(enable) = self.run.common.wasm.unknown_exports_allow {
            linker.allow_unknown_exports(enable);
        }

        let host = Host {
            wasi: self.build_wasi_ctx()?,
            wasi_threads: None,
            wasi_nn: None,
            limits: StoreLimits::default(),
            #[cfg(feature = "profiling")]
            guest_profiler: None,
        };
        wasi_common::sync::add_to_linker(&mut linker, |host| &mut host.wasi)?;

        let mut store = Store::new(&engine, host);

        if self.run.common.wasi.preview2 == Some(true) {
            bail!("cannot enable preview 2 with legacy implementation");
        }

        if self.run.common.wasi.nn == Some(true) {
            #[cfg(not(feature = "wasi-nn"))]
            {
                bail!("Cannot enable wasi-nn when the binary is not compiled with this feature.");
            }
            #[cfg(feature = "wasi-nn")]
            {
                wasmtime_wasi_nn::witx::add_to_linker(&mut linker, |host| {
                    Arc::get_mut(host.wasi_nn.as_mut().expect("WasiNnCtx present"))
                        .expect("wasi-nn is not implemented with multi-threading support")
                })?;
                store.data_mut().wasi_nn = Some(Arc::new(self.build_nn_ctx()?));
            }
        }

        if self.run.common.wasi.threads == Some(true) {
            #[cfg(not(feature = "wasi-threads"))]
            {
                bail!(
                    "Cannot enable wasi-threads when the binary is not compiled with this feature"
                );
            }
            #[cfg(feature = "wasi-threads")]
            {
                wasmtime_wasi_threads::add_to_linker(&mut linker, &mut store, &main, |host| {
                    host.wasi_threads.as_ref().expect("WasiThreadCtx present")
                });
                store.data_mut().wasi_threads = Some(Arc::new(WasiThreadsCtx::new(
                    main.clone(),
                    Arc::new(linker.clone()),
                )?))
            }
        }

        if self.run.common.wasi.http == Some(true) {
            bail!("Cannot enable wasi-http for core wasm modules");
        }

        store.data_mut().limits = self.run.store_limits();
        store.limiter(|t| &mut t.limits);

        // If fuel has been configured, we want to add the configured
        // fuel amount to this store.
        if let Some(fuel) = self.run.common.wasm.fuel {
            store.set_fuel(fuel)?;
        }

        // Load the preload wasm modules.
        let mut modules = Vec::new();
        modules.push((String::new(), main.clone()));
        for (name, path) in self.preloads.iter() {
            // Read the wasm module binary either as `*.wat` or a raw binary
            let module = match self.run.load_run_target(&engine, path)? {
                RunTarget::Core(m) => m,
                #[cfg(feature = "component-model")]
                RunTarget::Component(_) => bail!("components cannot be loaded with `--preload`"),
            };
            modules.push((name.clone(), module.clone()));

            // Add the module's functions to the linker.
            #[cfg(feature = "cranelift")]
            linker.module(&mut store, name, &module).context(format!(
                "failed to process preload `{}` at `{}`",
                name,
                path.display()
            ))?;
            #[cfg(not(feature = "cranelift"))]
            bail!("support for --preload disabled at compile time");
        }

        // The main module might be allowed to have unknown imports, which
        // should be defined as traps:
        if self.run.common.wasm.unknown_imports_trap == Some(true) {
            #[cfg(feature = "cranelift")]
            linker.define_unknown_imports_as_traps(&main)?;
            #[cfg(not(feature = "cranelift"))]
            bail!("support for `unknown-imports-trap` disabled at compile time");
        }

        // ...or as default values.
        if self.run.common.wasm.unknown_imports_default == Some(true) {
            #[cfg(feature = "cranelift")]
            linker.define_unknown_imports_as_default_values(&main)?;
            #[cfg(not(feature = "cranelift"))]
            bail!("support for `unknown-imports-trap` disabled at compile time");
        }
        let finish_epoch_handler = self.setup_epoch_handler(&mut store, modules)?;

        let instance = linker.instantiate(&mut store, &main).context(format!(
            "failed to instantiate {:?}",
            self.module_and_args[0]
        ))?;

        // If `_initialize` is present, meaning a reactor, then invoke
        // the function.
        if let Some(func) = instance.get_func(&mut store, "_initialize") {
            func.typed::<(), ()>(&store)?.call(&mut store, ())?;
        }

        // Look for the specific function provided or otherwise look for
        // "" or "_start" exports to run as a "main" function.
        let func = if let Some(name) = &self.invoke {
            Some(
                instance
                    .get_func(&mut store, name)
                    .ok_or_else(|| anyhow!("no func export named `{}` found", name))?,
            )
        } else {
            instance
                .get_func(&mut store, "")
                .or_else(|| instance.get_func(&mut store, "_start"))
        };

        let result = match func {
            Some(func) => self.invoke_func(&mut store, func),
            None => Ok(()),
        };

        finish_epoch_handler(&mut store);

        match result.with_context(|| {
            format!(
                "failed to run main module `{}`",
                self.module_and_args[0].to_string_lossy()
            )
        }) {
            Ok(()) => Ok(()),
            Err(e) => {
                // Exit the process if Wasmtime understands the error;
                // otherwise, fall back on Rust's default error printing/return
                // code.
                if store.data().preview2_ctx.is_some() {
                    if let Some(exit) = e
                        .downcast_ref::<wasmtime_wasi::I32Exit>()
                        .map(|c| c.process_exit_code())
                    {
                        std::process::exit(exit);
                    }
                    if e.is::<wasmtime::Trap>() {
                        eprintln!("Error: {e:?}");
                        cfg_if::cfg_if! {
                            if #[cfg(unix)] {
                                std::process::exit(rustix::process::EXIT_SIGNALED_SIGABRT);
                            } else if #[cfg(windows)] {
                                // https://docs.microsoft.com/en-us/cpp/c-runtime-library/reference/abort?view=vs-2019
                                std::process::exit(3);
                            }
                        }
                    }
                }
                Err(e)
            }
        }
    }
}

#[cfg(not(unix))]
fn ctx_set_listenfd(num_fd: usize, _builder: &mut WasiCtxBuilder) -> Result<usize> {
    Ok(num_fd)
}

#[cfg(unix)]
fn ctx_set_listenfd(mut num_fd: usize, builder: &mut WasiCtxBuilder) -> Result<usize> {
    use listenfd::ListenFd;

    for env in ["LISTEN_FDS", "LISTEN_FDNAMES"] {
        if let Ok(val) = std::env::var(env) {
            builder.env(env, &val)?;
        }
    }

    let mut listenfd = ListenFd::from_env();

    for i in 0..listenfd.len() {
        if let Some(stdlistener) = listenfd.take_tcp_listener(i)? {
            let _ = stdlistener.set_nonblocking(true)?;
            let listener = TcpListener::from_std(stdlistener);
            builder.preopened_socket((3 + i) as _, listener)?;
            num_fd = 3 + i;
        }
    }

    Ok(num_fd)
}
