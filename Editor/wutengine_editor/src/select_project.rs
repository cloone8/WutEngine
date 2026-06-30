//! Project selection flow

use std::path::Path;

use wutengine::entity::Entity;
use wutengine::runtime;
use wutengine::thread::TaskHandle;
use wutengine::window::Window;
use wutengine::window::WindowConfig;
use wutengine_egui::egui;

use crate::EditorWindowContainer;
use crate::EguiWindowContainer;
use crate::project::ProjectFile;
use crate::project::create::create_empty_project;
use crate::window::EditorWindow;

/// Spawns the entities that handle the "select project" flow, in which the user
/// can either pick an existing project or create a new one.
///
/// Once the user has selected a project, the editor will be restarted targeting that project
pub(crate) fn select_project() {
    log::info!("Selecting project to open");

    let initial_window_title = "Select a project - WutEngine Editor".to_string();
    let initial_window_size = (1280, 720);

    let initial_window = Window::create(WindowConfig {
        title: Some(initial_window_title.clone()),
        resizable: true,
        size: initial_window_size,
        fullscreen: None,
        ..Default::default()
    });

    let main_editor_window_entity = Entity::spawn_transformless("Main Editor Window");
    let main_editor_window_container = EguiWindowContainer::new(Some(initial_window));

    main_editor_window_entity.add_component(main_editor_window_container);

    let select_project_window = EditorWindowContainer::new(SelectProjectWindow {
        new_project_name: String::new(),
        open_project_task: None,
        create_project_task: None,
    });

    main_editor_window_entity.add_component(select_project_window);
}

struct SelectProjectWindow {
    new_project_name: String,
    open_project_task: Option<TaskHandle<Option<rfd::FileHandle>>>,
    create_project_task: Option<TaskHandle<Option<rfd::FileHandle>>>,
}

impl SelectProjectWindow {
    fn check_async_tasks(&mut self) {
        if let Some(result) = TaskHandle::get_if_started_and_ready(&mut self.open_project_task)
            && let Some(project_file) = result
        {
            open_project(project_file.path());
        }

        if let Some(result) = TaskHandle::get_if_started_and_ready(&mut self.create_project_task)
            && let Some(project_dir) = result
        {
            match create_empty_project(&self.new_project_name, project_dir.path()) {
                Ok(pf) => {
                    open_project(pf);
                }
                Err(e) => {
                    log::error!("Failed to create new project: {e}");
                }
            }
        }
    }
}

impl EditorWindow for SelectProjectWindow {
    fn show(&mut self, ui: &mut wutengine_egui::egui::Ui) {
        self.check_async_tasks();

        egui::CentralPanel::default_margins().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Select a project");

                if ui.button("Open...").clicked() && self.open_project_task.is_none() {
                    self.open_project_task = Some(pick_project_file());
                }

                ui.separator();

                egui::TextEdit::singleline(&mut self.new_project_name)
                    .hint_text("Project name...")
                    .show(ui);

                if ui.button("Create...").clicked() && self.create_project_task.is_none() {
                    self.create_project_task = Some(wutengine::thread::run_async(
                        rfd::AsyncFileDialog::new().pick_folder(),
                    ));
                }
            });
        });
    }
}

fn pick_project_file() -> TaskHandle<Option<rfd::FileHandle>> {
    wutengine::thread::run_async(
        rfd::AsyncFileDialog::new()
            .add_filter("WutEngine Project", &["we-project"])
            .pick_file(),
    )
}

fn open_project(project_file: impl AsRef<Path>) {
    let project_file = project_file.as_ref();

    if ProjectFile::from_disk(project_file).is_err() {
        eprintln!("Corrupt project file");
        return;
    }

    let cur_exe = std::env::current_exe().unwrap();
    let mut args = std::env::args_os().skip(1).collect::<Vec<_>>();

    args.push(project_file.as_os_str().to_os_string());

    #[allow(clippy::zombie_processes, reason = "We're starting a new editor")]
    std::process::Command::new(cur_exe)
        .args(args)
        .spawn()
        .unwrap();

    runtime::exit();
}
