use crate::preview2::WasiView;

wasmtime::component::bindgen!({
    world: "wasi:cli/command",
    tracing: true,
    async: true,
    with: {
       "wasi:filesystem/types": crate::preview2::bindings::filesystem::types,
       "wasi:filesystem/preopens": crate::preview2::bindings::filesystem::preopens,
       "wasi:sockets/tcp": crate::preview2::bindings::sockets::tcp,
       "wasi:clocks/monotonic_clock": crate::preview2::bindings::clocks::monotonic_clock,
       "wasi:io/poll": crate::preview2::bindings::io::poll,
       "wasi:io/streams": crate::preview2::bindings::io::streams,
       "wasi:clocks/wall_clock": crate::preview2::bindings::clocks::wall_clock,
       "wasi:random/random": crate::preview2::bindings::random::random,
       "wasi:cli/environment": crate::preview2::bindings::cli::environment,
       "wasi:cli/exit": crate::preview2::bindings::cli::exit,
       "wasi:cli/stdin": crate::preview2::bindings::cli::stdin,
       "wasi:cli/stdout": crate::preview2::bindings::cli::stdout,
       "wasi:cli/stderr": crate::preview2::bindings::cli::stderr,
       "wasi:cli/terminal-input": crate::preview2::bindings::cli::terminal_input,
       "wasi:cli/terminal-output": crate::preview2::bindings::cli::terminal_output,
       "wasi:cli/terminal-stdin": crate::preview2::bindings::cli::terminal_stdin,
       "wasi:cli/terminal-stdout": crate::preview2::bindings::cli::terminal_stdout,
       "wasi:cli/terminal-stderr": crate::preview2::bindings::cli::terminal_stderr,
    },
});

pub trait AsWasi: Send + 'static {
    fn as_wasi(&mut self) -> WasiView<'_>;
}
macro_rules! impl_as_wasi {
    ($p:path) => {
        impl<T: AsWasi> $p for T {
            type Ctx<'a> = WasiView<'a> where T: 'a;
            fn as_host(&mut self) -> WasiView<'_> {
                self.as_wasi()
            }
        }
    };
}

impl_as_wasi!(crate::preview2::bindings::clocks::wall_clock::AsHost);
impl_as_wasi!(crate::preview2::bindings::clocks::monotonic_clock::AsHost);
impl_as_wasi!(crate::preview2::bindings::filesystem::types::AsHost);
impl_as_wasi!(crate::preview2::bindings::filesystem::preopens::AsHost);
impl_as_wasi!(crate::preview2::bindings::io::error::AsHost);
impl_as_wasi!(crate::preview2::bindings::io::poll::AsHost);
impl_as_wasi!(crate::preview2::bindings::io::streams::AsHost);
impl_as_wasi!(crate::preview2::bindings::random::random::AsHost);
impl_as_wasi!(crate::preview2::bindings::random::insecure::AsHost);
impl_as_wasi!(crate::preview2::bindings::random::insecure_seed::AsHost);
impl_as_wasi!(crate::preview2::bindings::cli::exit::AsHost);
impl_as_wasi!(crate::preview2::bindings::cli::environment::AsHost);
impl_as_wasi!(crate::preview2::bindings::cli::stdin::AsHost);
impl_as_wasi!(crate::preview2::bindings::cli::stdout::AsHost);
impl_as_wasi!(crate::preview2::bindings::cli::stderr::AsHost);
impl_as_wasi!(crate::preview2::bindings::cli::terminal_input::AsHost);
impl_as_wasi!(crate::preview2::bindings::cli::terminal_output::AsHost);
impl_as_wasi!(crate::preview2::bindings::cli::terminal_stdin::AsHost);
impl_as_wasi!(crate::preview2::bindings::cli::terminal_stdout::AsHost);
impl_as_wasi!(crate::preview2::bindings::cli::terminal_stderr::AsHost);
impl_as_wasi!(crate::preview2::bindings::sockets::tcp::AsHost);
impl_as_wasi!(crate::preview2::bindings::sockets::tcp_create_socket::AsHost);
impl_as_wasi!(crate::preview2::bindings::sockets::udp::AsHost);
impl_as_wasi!(crate::preview2::bindings::sockets::udp_create_socket::AsHost);
impl_as_wasi!(crate::preview2::bindings::sockets::instance_network::AsHost);
impl_as_wasi!(crate::preview2::bindings::sockets::network::AsHost);
impl_as_wasi!(crate::preview2::bindings::sockets::ip_name_lookup::AsHost);

impl_as_wasi!(crate::preview2::bindings::sync_io::filesystem::types::AsHost);
impl_as_wasi!(crate::preview2::bindings::sync_io::io::poll::AsHost);
impl_as_wasi!(crate::preview2::bindings::sync_io::io::streams::AsHost);

pub fn add_to_linker<T: AsWasi + Send>(
    l: &mut wasmtime::component::Linker<T>,
) -> anyhow::Result<()> {
    crate::preview2::bindings::clocks::wall_clock::add_to_linker(l)?;
    crate::preview2::bindings::clocks::monotonic_clock::add_to_linker(l)?;
    crate::preview2::bindings::filesystem::types::add_to_linker(l)?;
    crate::preview2::bindings::filesystem::preopens::add_to_linker(l)?;
    crate::preview2::bindings::io::error::add_to_linker(l)?;
    crate::preview2::bindings::io::poll::add_to_linker(l)?;
    crate::preview2::bindings::io::streams::add_to_linker(l)?;
    crate::preview2::bindings::random::random::add_to_linker(l)?;
    crate::preview2::bindings::random::insecure::add_to_linker(l)?;
    crate::preview2::bindings::random::insecure_seed::add_to_linker(l)?;
    crate::preview2::bindings::cli::exit::add_to_linker(l)?;
    crate::preview2::bindings::cli::environment::add_to_linker(l)?;
    crate::preview2::bindings::cli::stdin::add_to_linker(l)?;
    crate::preview2::bindings::cli::stdout::add_to_linker(l)?;
    crate::preview2::bindings::cli::stderr::add_to_linker(l)?;
    crate::preview2::bindings::cli::terminal_input::add_to_linker(l)?;
    crate::preview2::bindings::cli::terminal_output::add_to_linker(l)?;
    crate::preview2::bindings::cli::terminal_stdin::add_to_linker(l)?;
    crate::preview2::bindings::cli::terminal_stdout::add_to_linker(l)?;
    crate::preview2::bindings::cli::terminal_stderr::add_to_linker(l)?;
    crate::preview2::bindings::sockets::tcp::add_to_linker(l)?;
    crate::preview2::bindings::sockets::tcp_create_socket::add_to_linker(l)?;
    crate::preview2::bindings::sockets::udp::add_to_linker(l)?;
    crate::preview2::bindings::sockets::udp_create_socket::add_to_linker(l)?;
    crate::preview2::bindings::sockets::instance_network::add_to_linker(l)?;
    crate::preview2::bindings::sockets::network::add_to_linker(l)?;
    crate::preview2::bindings::sockets::ip_name_lookup::add_to_linker(l)?;
    Ok(())
}

pub mod sync {

    wasmtime::component::bindgen!({
        world: "wasi:cli/command",
        tracing: true,
        async: false,
        with: {
           "wasi:filesystem/types": crate::preview2::bindings::sync_io::filesystem::types,
           "wasi:filesystem/preopens": crate::preview2::bindings::filesystem::preopens,
           "wasi:sockets/tcp": crate::preview2::bindings::sockets::tcp,
           "wasi:sockets/udp": crate::preview2::bindings::sockets::udp,
           "wasi:clocks/monotonic_clock": crate::preview2::bindings::clocks::monotonic_clock,
           "wasi:io/poll": crate::preview2::bindings::sync_io::io::poll,
           "wasi:io/streams": crate::preview2::bindings::sync_io::io::streams,
           "wasi:clocks/wall_clock": crate::preview2::bindings::clocks::wall_clock,
           "wasi:random/random": crate::preview2::bindings::random::random,
           "wasi:cli/environment": crate::preview2::bindings::cli::environment,
           "wasi:cli/exit": crate::preview2::bindings::cli::exit,
           "wasi:cli/stdin": crate::preview2::bindings::cli::stdin,
           "wasi:cli/stdout": crate::preview2::bindings::cli::stdout,
           "wasi:cli/stderr": crate::preview2::bindings::cli::stderr,
           "wasi:cli/terminal-input": crate::preview2::bindings::cli::terminal_input,
           "wasi:cli/terminal-output": crate::preview2::bindings::cli::terminal_output,
           "wasi:cli/terminal-stdin": crate::preview2::bindings::cli::terminal_stdin,
           "wasi:cli/terminal-stdout": crate::preview2::bindings::cli::terminal_stdout,
           "wasi:cli/terminal-stderr": crate::preview2::bindings::cli::terminal_stderr,
        },
    });

    pub fn add_to_linker<T: super::AsWasi + Send>(
        l: &mut wasmtime::component::Linker<T>,
    ) -> anyhow::Result<()> {
        crate::preview2::bindings::clocks::wall_clock::add_to_linker(l)?;
        crate::preview2::bindings::clocks::monotonic_clock::add_to_linker(l)?;
        crate::preview2::bindings::sync_io::filesystem::types::add_to_linker(l)?;
        crate::preview2::bindings::filesystem::preopens::add_to_linker(l)?;
        crate::preview2::bindings::io::error::add_to_linker(l)?;
        crate::preview2::bindings::sync_io::io::poll::add_to_linker(l)?;
        crate::preview2::bindings::sync_io::io::streams::add_to_linker(l)?;
        crate::preview2::bindings::random::random::add_to_linker(l)?;
        crate::preview2::bindings::random::insecure::add_to_linker(l)?;
        crate::preview2::bindings::random::insecure_seed::add_to_linker(l)?;
        crate::preview2::bindings::cli::exit::add_to_linker(l)?;
        crate::preview2::bindings::cli::environment::add_to_linker(l)?;
        crate::preview2::bindings::cli::stdin::add_to_linker(l)?;
        crate::preview2::bindings::cli::stdout::add_to_linker(l)?;
        crate::preview2::bindings::cli::stderr::add_to_linker(l)?;
        crate::preview2::bindings::cli::terminal_input::add_to_linker(l)?;
        crate::preview2::bindings::cli::terminal_output::add_to_linker(l)?;
        crate::preview2::bindings::cli::terminal_stdin::add_to_linker(l)?;
        crate::preview2::bindings::cli::terminal_stdout::add_to_linker(l)?;
        crate::preview2::bindings::cli::terminal_stderr::add_to_linker(l)?;
        crate::preview2::bindings::sockets::tcp::add_to_linker(l)?;
        crate::preview2::bindings::sockets::tcp_create_socket::add_to_linker(l)?;
        crate::preview2::bindings::sockets::udp::add_to_linker(l)?;
        crate::preview2::bindings::sockets::udp_create_socket::add_to_linker(l)?;
        crate::preview2::bindings::sockets::instance_network::add_to_linker(l)?;
        crate::preview2::bindings::sockets::network::add_to_linker(l)?;
        crate::preview2::bindings::sockets::ip_name_lookup::add_to_linker(l)?;
        Ok(())
    }
}
