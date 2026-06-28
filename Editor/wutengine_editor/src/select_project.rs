//! Project selection flow

use std::io::prelude::Write;
use std::path::PathBuf;

use wutengine::entity::Entity;
use wutengine::runtime;
use wutengine::window::Window;
use wutengine::window::WindowConfig;
use wutengine_egui::egui;

use crate::EditorWindowContainer;
use crate::EguiWindowContainer;
use crate::project::ProjectFile;
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
    });

    main_editor_window_entity.add_component(select_project_window);
}

struct SelectProjectWindow {
    new_project_name: String,
}

impl EditorWindow for SelectProjectWindow {
    fn show(&mut self, ui: &mut wutengine_egui::egui::Ui) {
        egui::CentralPanel::default_margins().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Select a project");

                if ui.button("Open...").clicked()
                    && let Some(project_file) = pick_project_file()
                {
                    open_project(project_file);
                }

                ui.separator();

                egui::TextEdit::singleline(&mut self.new_project_name)
                    .hint_text("Project name...")
                    .show(ui);

                if ui.button("Create...").clicked()
                    && let Some(project_file) = create_project_file(&self.new_project_name)
                {
                    let project_file_content =
                        serde_json::to_string_pretty(&ProjectFile::new()).unwrap();
                    let mut file = std::fs::File::create_new(&project_file).unwrap();

                    file.write_all(&project_file_content.into_bytes()).unwrap();

                    open_project(project_file);
                }
            });
        });
    }
}

fn create_project_file(project_name: &str) -> Option<PathBuf> {
    let parent_folder = rfd::FileDialog::new().pick_folder()?;

    let project_folder = parent_folder.join(project_name);

    std::fs::create_dir(&project_folder).unwrap();

    Some(project_folder.join(format!("{project_name}.we-project")))
}

fn pick_project_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("WutEngine Project", &["we-project"])
        .pick_file()
}

fn open_project(project_file: PathBuf) {
    if ProjectFile::from_disk(&project_file).is_err() {
        eprintln!("Corrupt project file");
        return;
    }

    let cur_exe = std::env::current_exe().unwrap();
    let mut args = std::env::args_os().skip(1).collect::<Vec<_>>();

    args.push(project_file.into_os_string());

    #[allow(clippy::zombie_processes, reason = "We're starting a new editor")]
    std::process::Command::new(cur_exe)
        .args(args)
        .spawn()
        .unwrap();

    runtime::exit();
}
