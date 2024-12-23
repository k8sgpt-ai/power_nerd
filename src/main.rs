use eframe::egui;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use k8sgpt_ai_k8sgpt_community_neoeinstein_prost::schema::v1::{AnalyzeRequest, AnalyzeResponse};
use tokio::runtime::Runtime;
use k8sgpt_ai_k8sgpt_community_neoeinstein_tonic::schema::v1::tonic::server_analyzer_service_client::ServerAnalyzerServiceClient;
use power_nerd::BACKEND_TYPES;

struct MyApp {
    // Sender/Receiver for async notifications.
    loading_tx: Sender<bool>,
    loading_rx: Receiver<bool>,
    connect_error_tx: Sender<String>,
    connect_error_rx: Receiver<String>,
    response_tx: Sender<AnalyzeResponse>,
    response_rx: Receiver<AnalyzeResponse>,
    // App state
    is_loading: bool,
    response: AnalyzeResponse,
    backend: String,
    error: String,
    // filter array
    selected_filter: String,
    explain: bool
}
fn main() {
    let rt = Runtime::new().expect("Unable to create Runtime");

    // Enter the runtime so that `tokio::spawn` is available immediately.
    let _enter = rt.enter();

    // Execute the runtime in its own thread.
    // The future doesn't have to do anything. In this example, it just sleeps forever.
    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        })
    });

    // Run the GUI in the main thread.
    let _ = eframe::run_native(
        "PowerNerd",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    );
}

impl Default for MyApp {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let (loading_tx, loading_rx) = std::sync::mpsc::channel();
        let (response_tx, response_rx):  (Sender<AnalyzeResponse>, Receiver<AnalyzeResponse>)= std::sync::mpsc::channel();
        let (connect_error_tx, connect_error_rx):  (Sender<String>, Receiver<String>)= std::sync::mpsc::channel();
        Self {
            loading_tx,
            loading_rx,
            connect_error_tx,
            connect_error_rx,
            response_tx: tx,
            response_rx: rx,
            is_loading: false,
            backend:  "openai".to_string(),
            error: Default::default(),
            response: Default::default(),
            selected_filter: "".to_string(),
            explain: true,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update the counter with the async response.

        if let Ok(loading) = self.loading_rx.try_recv() {
            self.is_loading = loading;
        }
        if let Ok(response) = self.response_rx.try_recv() {
            self.response = response;
        }
        if let Ok(error) = self.connect_error_rx.try_recv() {
            self.error = error;
        }
        // Error Popup window ---------------------------------------------------------------------
        if !self.error.is_empty() {
            egui::Window::new("Error")
                .open(&mut true)
                .show(ctx, |ui| {
                    ui.label(&self.error);
                });
            // close window on click
            if ctx.input(|i| i.pointer.any_click()) {
                self.error = "".to_string();
            }
        }
        // ----------------------------------------------------------------------------------------
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Backend Type:");
                egui::ComboBox::from_id_source("backend_type")
                    .selected_text(format!("{:?}", self.backend))
                    .show_ui(ui, |ui| {
                        for backend in BACKEND_TYPES {
                            ui.selectable_value(&mut self.backend, backend.to_string(), format!("{:?}", backend));
                        }
                    });
                // checkbox for filter types FILTER_TYPES
                ui.label("Filter Type:");
                egui::ComboBox::from_id_source("filter_type")
                    .selected_text(format!("{:?}", self.selected_filter))
                    .show_ui(ui, |ui| {
                        for filter in power_nerd::FILTER_TYPES {

                            // if it's set to None make it empty
                            ui.selectable_value(&mut self.selected_filter, filter.to_string(), format!("{:?}", filter));
                            if filter.contains("None") {
                                self.selected_filter = "".to_string();
                            }

                        }
                    });
            });
            // explain checkbox
            ui.checkbox(&mut self.explain, "Explain");
            // if explain is unchecked capture in the self.explain bool

        });
        // ----------------------------------------------------------------------------------------
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.is_loading {
                ui.label("Analyzing...");
                ui.spinner();
            }
            if !self.is_loading && ui.button("Analyze").clicked() {
                self.is_loading = true;
                send_req(self.loading_tx.clone(), self.backend.clone(),self.selected_filter.clone(),
                         self.explain,self.response_tx.clone(),self.connect_error_tx.clone(), ctx.clone());
            }
            if !self.response.results.is_empty() {
                // Display the results
                ui.label("Results:");
                // Print results into scrollable area
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for result in &self.response.results {

                        // convert result.error into a string
                        let mut error_message: Vec<String> = vec![];
                        for error in &result.error {
                            error_message.push(error.text.clone());
                        }
                        // convert vec to string
                        let error_message = error_message.join("\n");
                        ui.label(egui::RichText::new(format!("{}:\n", result.name)).heading().color(egui::Color32::from_rgb(255, 255, 255)));
                        ui.label(egui::RichText::new(format!("{}\n", error_message)).color(egui::Color32::from_rgb(252, 61, 3)));
                        ui.label(egui::RichText::new(format!("{}\n", result.details)).heading().color(egui::Color32::from_rgb(50, 141, 168)));
                    }
                });
            }
        });
    }
}

fn send_req(loading_tx: Sender<bool>,backend: String, selected_filter: String, explain: bool,
            response_tx: Sender<AnalyzeResponse>, connect_error_tx: Sender<String>,  ctx: egui::Context) {
    tokio::spawn(async move {

        let client = ServerAnalyzerServiceClient::connect("http://localhost:8080").await;
        if client.is_err() {
            connect_error_tx.send("Error connecting to server".to_string()).unwrap();
            loading_tx.send(false).unwrap();
            return;
        }

        // filter array
        let mut filters = vec![];
        if !selected_filter.is_empty() {
            filters.push(selected_filter);
        }

        let request = tonic::Request::new(AnalyzeRequest {
            backend,
            namespace: "".to_string(),
            explain,
            anonymize: false,
            nocache: false,
            language: "".to_string(),
            max_concurrency: 0,
            output: "".to_string(),
            filters,
            label_selector: "".to_string(),
        });
        let response = client.unwrap().analyze(request).await.unwrap();
        loading_tx.send(false).unwrap();

        response_tx.send(response.into_inner()).unwrap();
        ctx.request_repaint();
    });
}
