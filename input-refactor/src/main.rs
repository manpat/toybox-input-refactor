use toybox::prelude::*;


struct ContextGroups {
	standard_mouse_group: input::ContextGroupID,
	normalized_mouse_group: input::ContextGroupID,
	preserve_aspect_mouse_group: input::ContextGroupID,
}


fn main() -> Result<(), Box<dyn Error>> {
	std::env::set_var("RUST_BACKTRACE", "1");

	let mut engine = toybox::Engine::new("input-refactor")?;
	engine.imgui.set_visible(true);
	engine.imgui.set_input_enabled(true);

	// Build context groups
	let standard_mouse_group = engine.input.new_context_group("Window Mouse");
	let normalized_mouse_group = engine.input.new_context_group("Normalized Mouse");
	let preserve_aspect_mouse_group = engine.input.new_context_group("PreserveAspect Mouse");

	engine.input.set_context_group_active(standard_mouse_group, true);
	engine.input.set_context_group_active(normalized_mouse_group, false);
	engine.input.set_context_group_active(preserve_aspect_mouse_group, false);

	// Build contexts
	let mut context_builder = engine.input.new_context("Relative Mouse");
	context_builder.new_mouse("", input::MouseSpace::Window, 1.0);
	context_builder.set_context_group(standard_mouse_group);
	let relative_mouse_context = context_builder.build();

	let mut context_builder = engine.input.new_context("Absolute Mouse");
	context_builder.new_pointer("", input::MouseSpace::Window);
	context_builder.set_context_group(standard_mouse_group);
	let absolute_mouse_context = context_builder.build();


	let mut context_builder = engine.input.new_context("Relative Mouse (Normalized)");
	context_builder.new_mouse("", input::MouseSpace::Normalized, 1.0);
	context_builder.set_context_group(normalized_mouse_group);
	let relative_mouse_context_normalized = context_builder.build();

	let mut context_builder = engine.input.new_context("Absolute Mouse (Normalized)");
	context_builder.new_pointer("", input::MouseSpace::Normalized);
	context_builder.set_context_group(normalized_mouse_group);
	let absolute_mouse_context_normalized = context_builder.build();


	let mut context_builder = engine.input.new_context("Relative Mouse (PreserveAspect)");
	context_builder.new_mouse("", input::MouseSpace::PreserveAspect, 1.0);
	context_builder.set_context_group(preserve_aspect_mouse_group);
	let relative_mouse_context_preserve_aspect = context_builder.build();

	let mut context_builder = engine.input.new_context("Absolute Mouse (PreserveAspect)");
	context_builder.new_pointer("", input::MouseSpace::PreserveAspect);
	context_builder.set_context_group(preserve_aspect_mouse_group);
	let absolute_mouse_context_preserve_aspect = context_builder.build();


	let mut context_builder = engine.input.new_context("Keyboard");
	let trigger_action = context_builder.new_trigger("Trigger", input::Scancode::Space);
	let state_action = context_builder.new_state("State", input::MouseButton::Left);

	let quit_application_action = context_builder.new_trigger("Quit", input::Scancode::Escape);
	let toggle_relative_mouse_action = context_builder.new_trigger("Toggle Relative Mouse", input::Scancode::R);
	let toggle_demo_window_action = context_builder.new_trigger("Toggle Imgui Demo", input::Scancode::D);
	let keyboard_context = context_builder.build();

	engine.input.enter_context(keyboard_context);
	engine.input.enter_context(absolute_mouse_context);
	engine.input.enter_context(absolute_mouse_context_normalized);
	engine.input.enter_context(absolute_mouse_context_preserve_aspect);

	// Set up state
	let mut relative_mouse = false;
	let mut show_demo_window = false;
	let mut state_tracking_tab = StateTrackingTab{
		context_view: ContextView::new(),
		new_button_events: Vec::new(),
		processed_events: Vec::new(),
		state_action,
		trigger_action,
	};

	let context_groups = ContextGroups {
		standard_mouse_group,
		normalized_mouse_group,
		preserve_aspect_mouse_group,
	};

	let mut mouse_test_tab = MouseTestTab::new(&mut engine, context_groups);


	'main: loop {
		engine.process_events();
		engine.gfx.render_state().clear(gfx::ClearMode::ALL);

		if engine.should_quit() {
			break 'main
		}

		let state = engine.input.frame_state().clone();
		if state.active(quit_application_action) {
			break 'main
		}

		if state.active(toggle_relative_mouse_action) {
			relative_mouse = !relative_mouse;
			engine.input.set_context_active(relative_mouse_context, relative_mouse);
			engine.input.set_context_active(relative_mouse_context_normalized, relative_mouse);
			engine.input.set_context_active(relative_mouse_context_preserve_aspect, relative_mouse);
		}

		if state.active(toggle_demo_window_action) {
			show_demo_window = !show_demo_window;
		}

		state_tracking_tab.update(&engine);

		{
			let ui = engine.imgui.frame();

			if show_demo_window {
				ui.show_demo_window(&mut show_demo_window);
			}

			if let Some(_main_menu) = ui.begin_main_menu_bar()
				&& let Some(_tabbar) = ui.tab_bar("##main_tabs")
			{
				if let Some(_tab) = ui.tab_item("Main") {
					state_tracking_tab.draw(ui, &engine);
				}

				if let Some(_tab) = ui.tab_item("Mouse") {
					mouse_test_tab.draw(ui, &mut engine.input);
				}
			}
		}

		engine.end_frame();
	}

	Ok(())
}


struct StateTrackingTab {
	context_view: ContextView,
	new_button_events: Vec<input::Button>,
	processed_events: Vec<&'static str>,
	state_action: input::ActionID,
	trigger_action: input::ActionID,
}

impl StateTrackingTab {
	fn update(&mut self, engine: &toybox::Engine) {
		let state = engine.input.frame_state();

		if state.active(self.trigger_action) {
			self.processed_events.push("Trigger");
		}

		if state.entered(self.state_action) {
			self.processed_events.push("State Entered");
		}

		if state.left(self.state_action) {
			self.processed_events.push("State Left");
		}

		self.new_button_events.extend(&engine.input.raw_state.new_buttons);
	}

	fn draw(&mut self, ui: &imgui::Ui<'_>, engine: &toybox::Engine) {
		let state = engine.input.frame_state();
		let raw = &engine.input.raw_state;

		let width = 250.0;
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
			for button in self.new_button_events.iter().rev() {
				ui.text(format!("{button:?}"));
			}
		}

		if let Some(_window) = new_window("Frame State", false) {
			if state.active(self.state_action) {
				ui.label_text("State", "Active");
			} else {
				ui.label_text("State", "Inactive");
			}

			for event in self.processed_events.iter().rev() {
				ui.text(event);
			}
		}

		if let Some(_window) = new_window("Contexts##state_tracking_contexts", true) {
			self.context_view.draw(ui, &engine.input);
		}
	}
}


struct MouseTestTab {
	context_groups: ContextGroups,
}

impl MouseTestTab {
	fn new(_engine: &mut toybox::Engine, context_groups: ContextGroups) -> MouseTestTab {
		// TODO: visual feedback for mouse input

		MouseTestTab {
			context_groups,
		}
	}

	fn draw(&mut self, ui: &imgui::Ui<'_>, input: &mut input::InputSystem) {
		if let Some(_window) = imgui::Window::new("Contexts##mouse_test_contexts")
			.size([300.0, -1.0], imgui::Condition::Once)
			.position([0.0, 30.0], imgui::Condition::Appearing)
			.begin(ui)
		{
			if ui.button("Window") {
				input.set_context_group_active(self.context_groups.standard_mouse_group, true);
				input.set_context_group_active(self.context_groups.normalized_mouse_group, false);
				input.set_context_group_active(self.context_groups.preserve_aspect_mouse_group, false);
			}

			ui.same_line();
			if ui.button("Normalized") {
				input.set_context_group_active(self.context_groups.standard_mouse_group, false);
				input.set_context_group_active(self.context_groups.normalized_mouse_group, true);
				input.set_context_group_active(self.context_groups.preserve_aspect_mouse_group, false);
			}

			ui.same_line();
			if ui.button("Preserve Aspect") {
				input.set_context_group_active(self.context_groups.standard_mouse_group, false);
				input.set_context_group_active(self.context_groups.normalized_mouse_group, false);
				input.set_context_group_active(self.context_groups.preserve_aspect_mouse_group, true);
			}


			let state = input.frame_state();

			for context in input.contexts() {
				if let Some((action, action_id)) = context.mouse_action()
					&& input.is_context_active(context.id)
				{
					let color = match state.active(action_id) {
						true => [1.0; 4],
						false => [0.6; 4],
					};

					let _style = ui.push_style_color(imgui::StyleColor::Text, color);

					let kind = action.kind;
					let name = &context.name;

					if let Some(Vec2{x, y}) = state.mouse(action_id) {
						ui.text(format!("{kind:?} '{name}' ({x}, {y})"));

					} else {
						ui.text(format!("{kind:?} '{name}'"));
					}
				}
			}
		}
	}
}


struct ContextView {
	selected_context_id: Option<input::ContextID>,
}

impl ContextView {
	fn new() -> ContextView {
		ContextView {
			selected_context_id: None,
		}
	}

	fn draw(&mut self, ui: &imgui::Ui<'_>, input: &input::InputSystem) {
		let state = input.frame_state();

		if let Some(_list) = imgui::ListBox::new("context_group_list")
			.size([-1.0, 0.0])
			.begin(ui)
		{
			for context_group in input.context_groups() {
				let color = match input.is_context_group_active(context_group.id) {
					true => [1.0; 4],
					false => [0.6; 4],
				};

				let _style = ui.push_style_color(imgui::StyleColor::Text, color);

				let label = format!("{} {:?}", context_group.name, context_group.id);
				ui.text(label);
			}
		}

		if let Some(_list) = imgui::ListBox::new("context_list")
			.size([-1.0, 0.0])
			.begin(ui)
		{
			for context in input.contexts() {
				let color = match input.is_context_active(context.id()) {
					true => [1.0; 4],
					false => [0.6; 4],
				};

				let _style = ui.push_style_color(imgui::StyleColor::Text, color);

				let label = format!("{} {:?}", context.name(), context.id());
				if imgui::Selectable::new(label)
					.selected(Some(context.id()) == self.selected_context_id)
					.build(ui)
				{
					self.selected_context_id = Some(context.id());
				}
			}
		}


		if let Some(selected_context_id) = self.selected_context_id
			&& let Some(context) = input.context(selected_context_id)
		{
			ui.label_text("Name", context.name());
			ui.separator();
			ui.label_text("ID", format!("{:?}", context.id()));
			ui.label_text("Priority", format!("{}", context.priority()));
			if let Some(context_group_id) = context.context_group_id {
				ui.label_text("Context Group", format!("{context_group_id:?}"));
			}
			ui.separator();

			if let Some(_list) = imgui::ListBox::new("action_list")
				.size([-1.0, 0.0])
				.begin(ui)
			{
				for (action, action_id) in context.actions().zip(context.action_ids()) {
					let color = match state.active(action_id) {
						true => [1.0; 4],
						false => [0.6; 4],
					};

					let _style = ui.push_style_color(imgui::StyleColor::Text, color);

					let kind = action.kind;
					let name = &action.name;

					if let Some(Vec2{x, y}) = state.mouse(action_id) {
						ui.text(format!("{kind:?} '{name}' ({x}, {y})"));

					} else {
						ui.text(format!("{kind:?} '{name}'"));
					}
				}
			}
		}
	}
}