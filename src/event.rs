//! The `event` module contains traits and structs to actually run your game mainloop
//! and handle top-level state, as well as handle input events such as keyboard
//! and mouse.
//!
//! If you don't want to do this, you can write your own mainloop and
//! get the necessary event machinery by calling
//! `context.sdl_context.event_pump()` on your `Context`.  You can
//! then call whatever SDL event methods you want on that.  This is
//! not particularly elegant and is not guarenteed to be stable (if,
//! for instance, we someday get rid of SDL2), but trying to wrap it
//! up more conveniently really ends up with the exact same interface.
//! See issue <https://github.com/ggez/ggez/issues/117> for
//! discussion.

/// A key code.
pub use sdl2::keyboard::Keycode;

/// A struct that holds the state of modifier buttons such as ctrl or shift.
pub use sdl2::keyboard::Mod;
/// A mouse button press.
pub use sdl2::mouse::MouseButton;
/// A struct containing the mouse state at a given instant.
pub use sdl2::mouse::MouseState;

/// A controller button.
pub use sdl2::controller::Button;
/// A controller axis.
pub use sdl2::controller::Axis;

use sdl2::event::Event::*;
use sdl2::event;
use sdl2::mouse;
use sdl2::keyboard;


use context::Context;
use GameResult;
use timer;

use std::time::Duration;




/// A trait defining event callbacks; your primary interface with
/// `ggez`'s event loop.  Have a type implement this trait and
/// override at least the update() and draw() methods, then pass it to
/// `event::run()` to run the game's mainloop.
///
/// The default event handlers do nothing, apart from
/// `key_down_event()`, which will by default exit the game if escape
/// is pressed.  Just override the methods you want to do things with.
pub trait EventHandler {
    /// Called upon each physics update to the game.
    /// This should be where the game's logic takes place.
    fn update(&mut self, ctx: &mut Context, dt: Duration) -> GameResult<()>;

    /// Called to do the drawing of your game.
    /// You probably want to start this with
    /// `graphics::clear()` and end it with
    /// `graphics::present()` and `timer::sleep_until_next_frame()`
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()>;

    fn mouse_button_down_event(&mut self, _button: mouse::MouseButton, _x: i32, _y: i32) {}

    fn mouse_button_up_event(&mut self, _button: mouse::MouseButton, _x: i32, _y: i32) {}

    fn mouse_motion_event(
        &mut self,
        _state: mouse::MouseState,
        _x: i32,
        _y: i32,
        _xrel: i32,
        _yrel: i32,
    ) {
    }

    fn mouse_wheel_event(&mut self, _x: i32, _y: i32) {}

    fn key_down_event(&mut self, _keycode: Keycode, _keymod: Mod, _repeat: bool) {}

    fn key_up_event(&mut self, _keycode: Keycode, _keymod: Mod, _repeat: bool) {}

    fn controller_button_down_event(&mut self, _btn: Button, _instance_id: i32) {}
    fn controller_button_up_event(&mut self, _btn: Button, _instance_id: i32) {}
    fn controller_axis_event(&mut self, _axis: Axis, _value: i16, _instance_id: i32) {}

    /// Called when the window is shown or hidden.
    fn focus_event(&mut self, _gained: bool) {}

    /// Called upon a quit event.  If it returns true,
    /// the game does not exit.
    fn quit_event(&mut self) -> bool {
        println!("Quitting game");
        false
    }

    /// Called when the user resizes the window.
    /// Is not called when you resize it yourself with
    /// `graphics::set_mode()` though.
    fn resize_event(&mut self, _ctx: &mut Context, _width: u32, _height: u32) {}
}

/// Runs the game's main loop, calling event callbacks on the given state
/// object as events occur.
///
/// It does not try to do any type of framerate limiting.  See the
/// documentation for the `timer` module for more info.
pub fn run<S>(ctx: &mut Context, state: &mut S) -> GameResult<()>
where
    S: EventHandler,
{
    {
        let mut event_pump = ctx.sdl_context.event_pump()?;

        let mut continuing = true;
        while continuing {
            ctx.timer_context.tick();

            for event in event_pump.poll_iter() {
                match event {
                    Quit { .. } => {
                        continuing = state.quit_event();
                        // println!("Quit event: {:?}", t);
                    }
                    KeyDown {
                        keycode,
                        keymod,
                        repeat,
                        ..
                    } => if let Some(key) = keycode {
                        if key == keyboard::Keycode::Escape {
                            ctx.quit()?;
                        } else {
                            state.key_down_event(key, keymod, repeat)
                        }
                    },
                    KeyUp {
                        keycode,
                        keymod,
                        repeat,
                        ..
                    } => if let Some(key) = keycode {
                        state.key_up_event(key, keymod, repeat)
                    },
                    MouseButtonDown {
                        mouse_btn, x, y, ..
                    } => state.mouse_button_down_event(mouse_btn, x, y),
                    MouseButtonUp {
                        mouse_btn, x, y, ..
                    } => state.mouse_button_up_event(mouse_btn, x, y),
                    MouseMotion {
                        mousestate,
                        x,
                        y,
                        xrel,
                        yrel,
                        ..
                    } => state.mouse_motion_event(mousestate, x, y, xrel, yrel),
                    MouseWheel { x, y, .. } => state.mouse_wheel_event(x, y),
                    ControllerButtonDown { button, which, .. } => {
                        state.controller_button_down_event(button, which)
                    }
                    ControllerButtonUp { button, which, .. } => {
                        state.controller_button_up_event(button, which)
                    }
                    ControllerAxisMotion {
                        axis, value, which, ..
                    } => state.controller_axis_event(axis, value, which),
                    Window {
                        win_event: event::WindowEvent::FocusGained,
                        ..
                    } => state.focus_event(true),
                    Window {
                        win_event: event::WindowEvent::FocusLost,
                        ..
                    } => state.focus_event(false),
                    Window {
                        win_event: event::WindowEvent::Resized(w, h),
                        ..
                    } => {
                        state.resize_event(ctx, w as u32, h as u32);
                    }
                    _ => {}
                }
            }

            let dt = timer::get_delta(ctx);
            state.update(ctx, dt)?;
            state.draw(ctx)?;
        }
    }

    Ok(())
}
