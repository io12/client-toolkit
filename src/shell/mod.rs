//! Unified shell surface handling
//!
//! This module provides an abstraction unifying the various iterations of
//! the shell surface protocols (`wl_shell`, `zxdg_shell_v6` and `xdg_shell`,
//! the current standard).
//!
//! This abstraction only manages the protocol part of shell surfaces. If you're
//! looking for a more battery-included abstraction for creating windows,
//! consider the `Window` type.
use wayland_client::protocol::{wl_output, wl_seat, wl_surface};
use wayland_client::Proxy;

use wayland_protocols::xdg_shell::client::xdg_toplevel;
pub use wayland_protocols::xdg_shell::client::xdg_toplevel::State;

use Shell;

mod wl;
mod xdg;
mod zxdg;

/// Possible events generated by a shell surface that you need to handle
#[derive(Clone, Debug)]
pub enum Event {
    /// The state of your window has been changed
    Configure {
        /// Optional new size for your shell surface
        ///
        /// This is the new size of the contents of your shell surface
        /// as suggested by the server. You can ignore it and choose
        /// a new size if you want better control on the possible
        /// sizes of your shell surface.
        ///
        /// In all cases, these events can be generated in large batches
        /// during an interactive resize, and you should buffer them before
        /// processing them. You only need to handle the last one of a batch.
        new_size: Option<(u32, u32)>,
        /// New combination of states of your window
        ///
        /// Typically tells you if your surface is active/inactive, maximized,
        /// etc...
        states: Vec<State>,
    },
    /// A close request has been received
    ///
    /// Most likely the user has clicked on the close button of the decorations
    /// or something equivalent
    Close,
}

pub(crate) fn create_shell_surface<Impl>(
    shell: &Shell,
    surface: &Proxy<wl_surface::WlSurface>,
    implem: Impl,
) -> Box<ShellSurface>
where
    Impl: FnMut(Event) + Send + 'static,
{
    match *shell {
        Shell::Wl(ref shell) => Box::new(wl::Wl::create(surface, shell, implem)) as Box<_>,
        Shell::Xdg(ref shell) => Box::new(xdg::Xdg::create(surface, shell, implem)) as Box<_>,
        Shell::Zxdg(ref shell) => Box::new(zxdg::Zxdg::create(surface, shell, implem)) as Box<_>,
    }
}

/// Trait abstracting over shell surface protocols
///
/// This trait's API is designed to reflect the behavior of the current standard
/// shell surface protocol: `xdg_shell`. Compatibility implementations are
/// provided for older protocols.
pub trait ShellSurface: Send + Sync {
    /// Resizes the shell surface
    fn resize(&self, seat: &Proxy<wl_seat::WlSeat>, serial: u32, edges: xdg_toplevel::ResizeEdge);
    /// Moves the shell surface
    fn move_(&self, seat: &Proxy<wl_seat::WlSeat>, serial: u32);
    /// Set the title of the shell surface
    fn set_title(&self, title: String);
    /// Set the app id of the shell surface
    fn set_app_id(&self, app_id: String);
    /// Make fullscreen
    fn set_fullscreen(&self, output: Option<&Proxy<wl_output::WlOutput>>);
    /// Unset fullscreen
    fn unset_fullscreen(&self);
    /// Maximize surface
    fn set_maximized(&self);
    /// Unmaximize surface
    fn unset_maximized(&self);
    /// Minimize surface
    fn set_minimized(&self);
    /// Set geometry
    fn set_geometry(&self, x: i32, y: i32, width: i32, height: i32);
    /// Set minimum surface size
    fn set_min_size(&self, size: Option<(i32, i32)>);
    /// Set maximum surface size
    fn set_max_size(&self, size: Option<(i32, i32)>);
    /// Retrive the `XdgToplevel` proxy if the underlying shell surface
    /// uses the `xdg_shell` protocol.
    ///
    /// This allows interactions with other protocol extensions, like
    /// `xdg_decoratins` for example.
    fn get_xdg(&self) -> Option<&Proxy<xdg_toplevel::XdgToplevel>>;
}
