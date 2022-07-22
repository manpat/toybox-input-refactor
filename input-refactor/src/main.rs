use toybox::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
	std::env::set_var("RUST_BACKTRACE", "1");

	let mut engine = toybox::Engine::new("input-refactor")?;
	engine.imgui.set_visible(true);
	engine.imgui.set_input_enabled(true);

	let mut new_button_events: Vec<input::Button> = Vec::new();
	let mut processed_events: Vec<&str> = Vec::new();
	let mut relative_mouse = false;


	let mut context_builder = engine.input.new_context("Relative Mouse");
	context_builder.new_mouse("", 1.0);
	let relative_mouse_context = context_builder.build();


	let mut context_builder = engine.input.new_context("Keyboard");
	let trigger_action = context_builder.new_trigger("Trigger", input::Scancode::Space);
	let state_action = context_builder.new_state("State", input::MouseButton::Left);

	let quit_application_action = context_builder.new_trigger("Quit", input::Scancode::Escape);
	let toggle_relative_mouse_action = context_builder.new_trigger("Toggle Relative Mouse", input::Scancode::R);

	let main_context = context_builder.build();
	engine.input.enter_context(main_context);


	'main: loop {
		engine.process_events();
		if engine.should_quit() {
			break 'main
		}

		if engine.input.frame_state().active(quit_application_action) {
			break 'main
		}

		if engine.input.frame_state().active(toggle_relative_mouse_action) {
			relative_mouse = !relative_mouse;
			engine.input.set_context_active(relative_mouse_context, relative_mouse);
		}

		let state = engine.input.frame_state();

		if state.active(trigger_action) {
			processed_events.push("Trigger");
		}
		if state.entered(state_action) {
			processed_events.push("State Entered");
		}
		if state.left(state_action) {
			processed_events.push("State Left");
		}

		{
			let ui = engine.imgui.frame();
			let raw = &engine.input.raw;

			let width = 220.0;
			let height = 500.0;

			if let Some(_window) = imgui::Window::new("Raw")
				.size([width, height], imgui::Condition::Always)
				.position([width*0.0 + 50.0, 50.0], imgui::Condition::Always)
				.no_inputs()
				.no_decoration()
				.begin(ui)
			{
				ui.text(format!("{raw:#?}"));
			}

			if let Some(_window) = imgui::Window::new("Raw Mouse")
				.size([width, height], imgui::Condition::Always)
				.position([width*1.0 + 50.0, 50.0], imgui::Condition::Always)
				.no_inputs()
				.no_decoration()
				.begin(ui)
			{
				let some_color = [1.0; 4];
				let none_color = [1.0, 0.0, 0.0, 1.0];

				let (abs_str, abs_color) = match raw.mouse_absolute {
					Some(Vec2i{x, y}) => (format!("{x} {y}"), some_color),
					None => ("None".into(), none_color),
				};

				let (rel_str, rel_color) = match raw.mouse_delta {
					Some(Vec2i{x, y}) => (format!("{x} {y}"), some_color),
					None => ("0 0".into(), none_color),
				};

				let (wheel_str, wheel_color) = match raw.wheel_delta {
					Some(delta) => (delta.to_string(), some_color),
					None => ("0".into(), none_color),
				};

				let _style = ui.push_style_color(imgui::StyleColor::Text, abs_color);
				ui.label_text("Absolute", abs_str);

				let _style = ui.push_style_color(imgui::StyleColor::Text, rel_color);
				ui.label_text("Relative", rel_str);

				let _style = ui.push_style_color(imgui::StyleColor::Text, wheel_color);
				ui.label_text("Wheel", wheel_str);
			}

			if let Some(_window) = imgui::Window::new("Buttons")
				.size([width, height], imgui::Condition::Always)
				.position([width*2.0 + 50.0, 50.0], imgui::Condition::Always)
				.no_inputs()
				.no_decoration()
				.begin(ui)
			{
				new_button_events.extend(&raw.new_buttons);

				for button in new_button_events.iter().rev() {
					ui.text(format!("{button:?}"));
				}
			}

			if let Some(_window) = imgui::Window::new("State")
				.size([width, height], imgui::Condition::Always)
				.position([width*3.0 + 50.0, 50.0], imgui::Condition::Always)
				.no_inputs()
				.no_decoration()
				.begin(ui)
			{
				if state.active(state_action) {
					ui.label_text("State", "Active");
				} else {
					ui.label_text("State", "Inactive");
				}

				for event in processed_events.iter().rev() {
					ui.text(event);
				}
			}
		}

		engine.gfx.render_state().clear(gfx::ClearMode::ALL);

		engine.end_frame();
	}

	Ok(())
}
