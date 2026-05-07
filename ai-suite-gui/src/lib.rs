mod app;
mod backend;
mod commands;
mod message;
mod settings;
mod theme;
mod ui;

use app::App;

pub fn run() -> anyhow::Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|error| anyhow::anyhow!("failed to build tokio runtime: {error}"))?;
    let handle = rt.handle().clone();
    let _rt = rt;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1060.0, 760.0])
            .with_min_inner_size([820.0, 560.0])
            .with_title("ai-suite"),
        ..Default::default()
    };

    eframe::run_native(
        "ai-suite",
        native_options,
        Box::new(move |_cc| Ok(Box::new(App::new(handle)) as Box<dyn eframe::App>)),
    )
    .map_err(|error| anyhow::anyhow!("failed to run GUI: {error}"))
}
