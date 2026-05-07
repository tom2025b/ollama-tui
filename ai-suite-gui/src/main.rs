mod app;
mod backend;

use app::App;

fn main() {
    // Build the tokio runtime before eframe takes over the main thread.
    // The `_rt` binding keeps it alive for the full duration of the window.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build tokio runtime");
    let handle = rt.handle().clone();
    let _rt = rt;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([860.0, 620.0])
            .with_title("ai-suite"),
        ..Default::default()
    };

    eframe::run_native(
        "ai-suite",
        native_options,
        Box::new(move |_cc| Ok(Box::new(App::new(handle)) as Box<dyn eframe::App>)),
    )
    .expect("Failed to run GUI");
}
