//! # promkit
//!
//! [![ci](https://github.com/ynqa/promkit/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/ynqa/promkit/actions/workflows/ci.yml)
//! [![docs.rs](https://img.shields.io/docsrs/promkit)](https://docs.rs/promkit)
//!
//! A toolkit for building your own interactive prompt in Rust.
//!
//! ## Getting Started
//!
//! Put the package in your `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! promkit = "0.3.0"
//! ```
//!
//! ## Features
//!
//! - Support cross-platform both UNIX and Windows owing to [crossterm](https://github.com/crossterm-rs/crossterm)
//! - Various building methods
//!   - Preset; Support for quickly setting up a UI by providing simple parameters.
//!     - [Readline](https://github.com/ynqa/promkit/tree/v0.3.0#readline)
//!     - [Confirm](https://github.com/ynqa/promkit/tree/v0.3.0#confirm)
//!     - [Password](https://github.com/ynqa/promkit/tree/v0.3.0#password)
//!     - [Select](https://github.com/ynqa/promkit/tree/v0.3.0#select)
//!     - [QuerySelect](https://github.com/ynqa/promkit/tree/v0.3.0#queryselect)
//!     - [Checkbox](https://github.com/ynqa/promkit/tree/v0.3.0#checkbox)
//!     - [Tree](https://github.com/ynqa/promkit/tree/v0.3.0#tree)
//!   - Combining various UI components.
//!     - They are provided with the same interface, allowing users to choose and
//!       assemble them according to their preferences.
//!   - (Upcoming) Stronger support to display yor own data structures.
//! - Versatile customization capabilities
//!   - Theme for designing the appearance of the prompt.
//!     - e.g. cursor, text
//!   - Validation for user input and error message construction.
//!
//! ## Examples/Demos
//!
//! See [here](https://github.com/ynqa/promkit/tree/v0.3.0#examplesdemos)
//!
//! ## Why *promkit*?
//!
//! Related libraries in this category include the following:
//! - [console-rs/dialoguer](https://github.com/console-rs/dialoguer)
//! - [mikaelmello/inquire](https://github.com/mikaelmello/inquire/tree/main/inquire)
//!
//! *promkit* offers several advantages over these libraries:
//!
//! ### Unified interface approach for UI components
//!
//! *promkit* takes a unified approach by having all of its components inherit the
//! same `Renderer` trait. This design choice enables users to seamlessly support
//! their custom data structures for display, similar to the relationships seen in
//! TUI projects like [ratatui-org/ratatui](https://github.com/ratatui-org/ratatui)
//! and
//! [EdJoPaTo/tui-rs-tree-widget](https://github.com/EdJoPaTo/tui-rs-tree-widget).
//! In other words, it's straightforward for anyone to display their own data
//! structures using widgets within promkit.  
//! In contrast, other libraries tend to treat each prompt as a mostly independent
//! entity. If you want to display a new data structure, you often have to build the
//! UI from scratch, which can be a time-consuming and less flexible process.
//!
//!   ```ignore
//!   pub trait Renderer {
//!       fn make_pane(&self, width: u16) -> Pane;
//!       fn handle_event(&mut self, event: &Event);
//!       fn postrun(&mut self);
//!   }
//!   ```
//!
//! ### Variety of Pre-built UI Preset Components
//!
//! One of the compelling reasons to choose *promkit* is its extensive range of pre-built UI preset components.
//! These presets allow developers to quickly implement various interactive prompts without the need to design and
//! build each component from scratch. The availability of these presets not only speeds up the development process
//! but also ensures consistency and reliability across different applications.
//! Here are some of the preset components available, see [Examples](#examplesdemos)
//!
//! ### Resilience to terminal resizing
//!
//! Performing operations that involve executing a command in one pane while
//! simultaneously opening a new pane is a common occurrence. During such operations,
//! if UI corruption is caused by resizing the terminal size, it may adversely affect
//! the user experience.  
//! Other libraries can struggle when the terminal is resized, making typing and
//! interaction difficult or impossible. For example:
//!
//!  - [(console-rs/dialoguer) Automatic re-render on terminal window resize](https://github.com/console-rs/dialoguer/issues/178)
//!
//! *promkit* introduces a step to align data with the screen size before rendering.
//! This approach ensures consistency in UI elements even when
//! the terminal size changes, providing a smoother user experience.
//!
//! ## License
//!
//! This project is licensed under the MIT License.
//! See the [LICENSE](https://github.com/ynqa/promkit/blob/main/LICENSE)
//! file for details.
//!
//! ## Stargazers over time
//! [![Stargazers over time](https://starchart.cc/ynqa/promkit.svg?variant=adaptive)](https://starchart.cc/ynqa/promkit)

pub use crossterm;
pub use serde_json;

mod core;
pub use core::*;
mod engine;
pub mod error;
mod grapheme;
pub mod keymap;
mod pane;
pub mod preset;
pub mod snapshot;
pub mod style;
mod terminal;
pub mod validate;

use std::{any::Any, io, sync::Once};

use crate::{
    crossterm::{
        cursor,
        event::{self, Event},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode},
    },
    engine::Engine,
    error::{Error, Result},
    pane::Pane,
    terminal::Terminal,
};

/// Represents the action to be taken after an event is processed.
///
/// This enum is used to determine how the `Prompt::run` method should proceed
/// after handling an event for a `Renderer` component.
///
/// - `Continue`: Indicates that the prompt should continue running and process further events.
/// - `Quit`: Signals that the prompt should stop running. If any of the `Renderer` components
///   returns `Quit`, a flag is set to indicate that the prompt should terminate. This allows
///   for a graceful exit when the user has completed their interaction with the prompt or when
///   an exit condition is met.
#[derive(Eq, PartialEq)]
pub enum EventAction {
    Continue,
    Quit,
}

pub type EventHandler<S> = fn(&mut S, &Event) -> Result<EventAction>;

/// A trait for objects that can be rendered in the terminal.
/// It requires the ability to create a pane, handle events,
/// and perform cleanup.
pub trait Renderer: AsAny {
    /// Creates a pane with the given width.
    fn make_pane(&self, width: u16) -> Pane;
    /// Handles terminal events.
    fn handle_event(&mut self, event: &Event) -> Result<EventAction>;
    /// Performs something (e.g. cleanup) after rendering is complete.
    fn postrun(&mut self);
}

/// A trait for casting objects to `Any`, allowing for dynamic typing.
pub trait AsAny {
    /// Returns `Any`.
    fn as_any(&self) -> &dyn Any;
}

/// `Evaluator` is defined using `dyn Fn` to leverage closures,
// enabling the capture and utilization of external variables within its scope.
// This design choice allows for the incorporation of validators or
// other context-specific data directly into the evaluation logic.
// Unlike static function pointers (`fn`),
// closures with `dyn Fn` can encapsulate their surrounding environment,
// offering a flexible solution for scenarios requiring access to external data or state.
type Evaluator = dyn Fn(&Event, &Vec<Box<dyn Renderer>>) -> Result<bool>;
type ResultProducer<T> = fn(&Vec<Box<dyn Renderer>>) -> Result<T>;

/// A core data structure to manage the hooks and state.
pub struct Prompt<T> {
    renderers: Vec<Box<dyn Renderer>>,
    evaluator: Box<Evaluator>,
    producer: ResultProducer<T>,
}

static ONCE: Once = Once::new();

impl<T> Drop for Prompt<T> {
    fn drop(&mut self) {
        execute!(io::stdout(), cursor::MoveToNextLine(1)).ok();
        execute!(io::stdout(), cursor::Show).ok();
        disable_raw_mode().ok();
    }
}

impl<T> Prompt<T> {
    /// Creates a new `Prompt` instance
    /// with specified renderers, evaluator, and producer functions.
    ///
    /// # Arguments
    ///
    /// * `renderers` - A vector of boxed objects implementing
    /// the `Renderer` trait.
    /// These are the UI components that will be rendered.
    /// * `evaluator` - A function that takes an event
    /// and the current state of renderer,
    /// returning a `Result<bool>` indicating
    /// whether the prompt is ready to produce an output.
    /// * `producer` - A function that takes the current state of renderer
    /// and returns a `Result<T>`, where `T` is the type of the output
    /// produced by the prompt.
    ///
    /// # Returns
    ///
    /// Returns a `Result` wrapping a new `Prompt` instance
    /// if successful, or an error if the creation fails.
    pub fn try_new<E>(
        renderers: Vec<Box<dyn Renderer>>,
        evaluator: E,
        producer: ResultProducer<T>,
    ) -> Result<Self>
    where
        E: Fn(&Event, &Vec<Box<dyn Renderer>>) -> Result<bool> + 'static,
    {
        Ok(Self {
            renderers,
            evaluator: Box::new(evaluator),
            producer,
        })
    }

    /// Runs the prompt, handling events and rendering UI components
    /// until an output is produced or an error occurs.
    ///
    /// This method initializes the terminal
    /// in raw mode, hides the cursor, and enters a loop to handle events.
    /// It continuously renders the UI components
    /// based on the current state and events until the evaluator function
    /// indicates that the prompt is ready to produce
    /// an output or an interrupt signal (e.g., Ctrl+C) is received.
    ///
    /// # Returns
    ///
    /// Returns a `Result<T>`, where `T` is the type of the output
    /// produced by the prompt, or an error if the prompt fails to run.
    pub fn run(&mut self) -> Result<T> {
        let mut engine = Engine::new(io::stdout());

        ONCE.call_once(|| {
            engine.clear().ok();
        });

        enable_raw_mode()?;
        execute!(io::stdout(), cursor::Hide)?;

        let mut terminal = Terminal::start_session(&mut engine)?;
        let size = engine.size()?;
        terminal.draw(
            &mut engine,
            self.renderers
                .iter()
                .map(|editor| editor.make_pane(size.0))
                .collect(),
        )?;

        loop {
            let ev = event::read()?;

            // This flow iterates through each renderer component,
            // handling the current event.
            // If a component signals to quit (EventAction::Quit),
            // it sets a flag to indicate the prompt should quit.
            // If an error occurs while handling the event,
            // it returns the error immediately.
            let mut should_quit = false;
            for renderer in &mut self.renderers {
                match renderer.handle_event(&ev) {
                    Ok(EventAction::Quit) => {
                        should_quit = true;
                        break;
                    }
                    Err(e) => return Err(e),
                    _ => (),
                }
            }

            let is_ready_for_output = (self.evaluator)(&ev, &self.renderers)?;

            if should_quit && is_ready_for_output {
                break;
            }

            let size = engine.size()?;
            terminal.draw(
                &mut engine,
                self.renderers
                    .iter()
                    .map(|editor| editor.make_pane(size.0))
                    .collect(),
            )?;
        }

        let ret = (self.producer)(&self.renderers);
        self.renderers.iter_mut().for_each(|editor| {
            editor.postrun();
        });
        ret
    }
}
