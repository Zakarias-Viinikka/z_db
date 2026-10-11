use eframe::egui;
use every_page_ever::MainPages::HomePage::HomePageState;

/*
 *** Egui requires a struct that implements eframe::App ***
 *
 * i don't think it cares how you actually drawing the thing, but it needs a struct that implements that
 *
 * ---
 *
 * eframe::App implements ui, which provides the ui signature, which is what's needed to actaully draw
 *
 * ---
 *
 *
 * egui::CentralPanel::default().show(ui, |ui| {
 *  // put code here to use ui to draw
 * });
 *
 * ---
 *
 * it might be unecessary but i put the code that actually
 * draws as a trait on the struct egui requires anyway.
 *
 *
 */

pub struct NecessaryStructForEgui {
    pub state: HomePageState,
}

impl eframe::App for NecessaryStructForEgui {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            let ctx = CtxForActuallyDrawing {
                ui,
                state: &mut self.state,
            };
            (Self::literally_draw_the_page)(ctx);
        });
    }

    fn on_exit(&mut self) {
        // The worker thread might still be running a check. Give it the
        // cancel signal; the guard's Drop will restore the file even if
        // the process exits right after.
        self.state.request_cancel();
    }
}

pub trait LiterallyDrawThePage {
    fn literally_draw_the_page(ctx: CtxForActuallyDrawing);
}

impl LiterallyDrawThePage for NecessaryStructForEgui {
    fn literally_draw_the_page(ctx: CtxForActuallyDrawing) {
        every_page_ever::MainPages::HomePage::view::draw(ctx.ui, ctx.state);
    }
}

pub struct CtxForActuallyDrawing<'a> {
    pub ui: &'a mut egui::Ui,
    pub state: &'a mut HomePageState,
}