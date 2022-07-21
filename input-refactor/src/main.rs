use toybox::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
	std::env::set_var("RUST_BACKTRACE", "1");

	let mut engine = toybox::Engine::new("input-refactor")?;
	engine.imgui.set_visible(true);
	engine.imgui.set_input_enabled(true);

	let mut new_button_events: Vec<input::raw::Button> = Vec::new();

	'main: loop {
		engine.process_events();

		if engine.should_quit() {
			break 'main
		}

		{
			let ui = engine.imgui.frame();
			let raw = &engine.input.raw;

			let width = 200.0;
			let height = 500.0;

			if let Some(_window) = imgui::Window::new("Input")
				.size([width, height], imgui::Condition::Always)
				.position([50.0, 50.0], imgui::Condition::Always)
				.no_inputs()
				.no_decoration()
				.begin(ui)
			{
				ui.text(format!("{raw:#?}"));
			}

			if let Some(_window) = imgui::Window::new("Blah")
				.size([width, height], imgui::Condition::Always)
				.position([250.0, 50.0], imgui::Condition::Always)
				.no_inputs()
				.no_decoration()
				.begin(ui)
			{
				ui.text(format!("{:#?}", raw.mouse_absolute));
				ui.text(format!("{:#?}", raw.active_buttons));
			}

			if let Some(_window) = imgui::Window::new("Buttons")
				.size([width, height], imgui::Condition::Always)
				.position([450.0, 50.0], imgui::Condition::Always)
				.no_inputs()
				.no_decoration()
				.begin(ui)
			{
				new_button_events.extend(&raw.new_buttons);

				imgui::ListBox::new("")
					.build_simple(
						ui,
						&mut 0,
						&new_button_events,
						&|v| format!("{v:?}").into()
					);
			}
		}

		engine.gfx.render_state().clear(gfx::ClearMode::ALL);

		engine.end_frame();
	}

	Ok(())
}
