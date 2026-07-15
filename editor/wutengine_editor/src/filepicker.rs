//! Asynchronous file/folder pick dialogs

use std::path::PathBuf;

use wutengine::runtime;
use wutengine::task::TaskHandle;

fn run_async<Fn, Fut, Out, FnAfter, Ret>(func: Fn, after: FnAfter) -> TaskHandle<Ret>
where
    Fn: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Out> + Send + 'static,
    Out: Send + 'static,
    FnAfter: FnOnce(Out) -> Ret + Send + 'static,
    Ret: Send + 'static,
{
    let pick_file_task = runtime::run_on_main_thread(func);

    wutengine::task::spawn_async(async move {
        let result = pick_file_task.get_async().await.await;

        let output = after(result);

        wutengine::runtime::request_frame();

        output
    })
}

pub(crate) fn pick_file(fd: rfd::AsyncFileDialog) -> TaskHandle<Option<PathBuf>> {
    run_async(
        move || fd.pick_file(),
        |filehandle| filehandle.map(|fh| fh.path().to_path_buf()),
    )
}

pub(crate) fn pick_files(fd: rfd::AsyncFileDialog) -> TaskHandle<Option<Vec<PathBuf>>> {
    run_async(
        move || fd.pick_files(),
        |filehandles| {
            filehandles.map(|fhs| fhs.into_iter().map(|fh| fh.path().to_path_buf()).collect())
        },
    )
}

pub(crate) fn pick_folder(fd: rfd::AsyncFileDialog) -> TaskHandle<Option<PathBuf>> {
    run_async(
        move || fd.pick_folder(),
        |filehandle| filehandle.map(|fh| fh.path().to_path_buf()),
    )
}

pub(crate) fn pick_folders(fd: rfd::AsyncFileDialog) -> TaskHandle<Option<Vec<PathBuf>>> {
    run_async(
        move || fd.pick_folders(),
        |filehandles| {
            filehandles.map(|fhs| fhs.into_iter().map(|fh| fh.path().to_path_buf()).collect())
        },
    )
}
