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

	let mut context_builder = engine.input.new_context("Absolute Mouse");
	context_builder.new_pointer("");
	let absolute_mouse_context = context_builder.build();


	let mut context_builder = engine.input.new_context("Keyboard");
	let trigger_action = context_builder.new_trigger("Trigger", input::Scancode::Space);
	let state_action = context_builder.new_state("State", input::MouseButton::Left);

	let quit_application_action = context_builder.new_trigger("Quit", input::Scancode::Escape);
	let toggle_relative_mouse_action = context_builder.new_trigger("Toggle Relative Mouse", input::Scancode::R);

	let main_context = context_builder.build();
	engine.input.enter_context(main_context);
	engine.input.enter_context(absolute_mouse_context);


	let mut selected_context_id = None;


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
			let raw = &engine.input.raw_state;

			let width = 220.0;
			let height = engine.gfx.backbuffer_size().y as f32 - 100.0;

			let mut x = 50.0;

			let mut new_window = |name, interactable| {
				let pos_x = x;
				x += width + 10.0;

				imgui::Window::new(name)
					.size([width, height], imgui::Condition::Always)
					.position([pos_x, 50.0], imgui::Condition::Always)
					.no_nav()
					.no_decoration()
					.mouse_inputs(interactable)
					.scroll_bar(interactable)
					.begin(ui)
			};

			if let Some(_window) = new_window("Raw", false) {
				ui.text(format!("{raw:#?}"));
			}

			if let Some(_window) = new_window("Raw Mouse", false) {
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

			if let Some(_window) = new_window("Button Presses", false) {
				new_button_events.extend(&raw.new_buttons);

				for button in new_button_events.iter().rev() {
					ui.text(format!("{button:?}"));
				}
			}

			if let Some(_window) = new_window("Frame State", false) {
				if state.active(state_action) {
					ui.label_text("State", "Active");
				} else {
					ui.label_text("State", "Inactive");
				}

				for event in processed_events.iter().rev() {
					ui.text(event);
				}
			}

			if let Some(_window) = new_window("Contexts", true) {
				let list = imgui::ListBox::new("context_list")
					.size([-1.0, 0.0])
					.begin(ui)
					.unwrap();

				for context in engine.input.contexts() {
					let color = match engine.input.is_context_active(context.id()) {
						true => [1.0; 4],
						false => [0.6; 4],
					};

					let _style = ui.push_style_color(imgui::StyleColor::Text, color);

					let label = format!("{} {:?}", context.name(), context.id());
					if imgui::Selectable::new(label)
						.selected(Some(context.id()) == selected_context_id)
						.build(ui)
					{
						selected_context_id = Some(context.id());
					}
				}

				list.end();


				if let Some(selected_context_id) = selected_context_id {
					let context = engine.input.contexts()
						.find(|ctx| ctx.id() == selected_context_id).unwrap();

					ui.label_text("Name", context.name());
					ui.separator();
					ui.label_text("ID", format!("{:?}", context.id()));
					ui.label_text("Priority", format!("{}", context.priority()));
					ui.separator();

					let list = imgui::ListBox::new("action_list")
						.size([-1.0, 0.0])
						.begin(ui)
						.unwrap();

					for (action, action_id) in context.actions().zip(context.action_ids()) {
						let color = match state.active(action_id) {
							true => [1.0; 4],
							false => [0.6; 4],
						};

						let _style = ui.push_style_color(imgui::StyleColor::Text, color);

						let kind = action.kind();
						let name = action.name();
						ui.text(format!("{kind:?} '{name}'"));
					}

					list.end();
				}
			}
		}

		engine.gfx.render_state().clear(gfx::ClearMode::ALL);

		engine.end_frame();
	}

	Ok(())
}
