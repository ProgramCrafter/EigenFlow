#![allow(unused_macros)]

use eframe::{App, Frame};
use egui::{RichText, Color32, CentralPanel, TopBottomPanel, ScrollArea, menu};



macro_rules! color {
    ($r:expr, $g:expr, $b:expr) => {
        Color32::from_rgb($r, $g, $b)
    }
}
macro_rules! frame {
    (margin: $margin:expr, fill: $r:literal $g:literal $b:literal) => {
        egui::Frame::none().inner_margin($margin).fill(Color32::from_rgb($r, $g, $b))
    }
}
macro_rules! text {
    ($text:expr) => {
        RichText::new($text).size(15.0)
    };
    ($text:expr, size $size:expr) => {
        RichText::new($text).size($size)
    };
    ($text:expr, color $c:expr) => {
        RichText::new($text).size(15.0).color($c)
    };
    ($text:expr, size $size:expr, color $c:expr) => {
        RichText::new($text).size($size).color($c)
    };
}
macro_rules! sized_button {
    (to $ui:expr, size $width:expr, $height:expr, text $comment:expr) => {
        $ui.add_sized([$width, $height], egui::Button::new($comment))
    };
    (to $ui:expr, size $width:expr, $height:expr, text $comment:expr, fill $c:expr) => {
        $ui.add_sized([$width, $height], egui::Button::new($comment).fill($c))
    };
    (to $ui:expr, height $height:expr, $($etc:tt)+) => {
        sized_button!(to $ui, size $ui.available_width(), $height, $($etc)+)
    };
}
macro_rules! label {
    (to $ui:expr, format ($($etc:tt)+)) => {
        $ui.label(text!(
          format!($($etc)+)
        ));
    };
    (to $ui:expr, $comment:expr) => {
        $ui.label($comment);
    };
}



fn eliminate_var(row_src: &Vec<f32>, row_eliminate: &mut Vec<f32>, var: usize) -> Result<(), ()> {
    if row_eliminate[var].abs() <= f32::EPSILON { return Ok(()); }
    if row_src[var].abs() <= f32::EPSILON { return Err(()); }
    
    let multiplier = -row_eliminate[var] / row_src[var];
    for (src, elim) in row_src.iter().zip(row_eliminate.iter_mut()) {
        *elim += *src * multiplier;
    }
    Ok(())
}

fn solve_lineq(equations: &mut Vec<Vec<f32>>) -> Result<(), ()> {
    // E[0][0] * x   + E[0][1] * y + ... + E[0][n] = 0
    // E[1][0] * x   + E[1][1] * y + ... + E[1][n] = 0
    // ...
    // E[n-1][0] * x + E[n-1][1] * y + . + E[n-1][n] = 0
    
    // \forall i, E[i][i] is nonzero
    
    let n = equations.len();
    
    for var in 0..n {
        let (top, bottom) = equations.split_at_mut(var + 1);
        let row_var = top.last().unwrap();
        for b in bottom {
            eliminate_var(row_var, b, var)?;
        }
    }
    
    // E[0][0] * x   + E[0][1] * y   + E[0][2] * z  + ... + E[0][n] = 0
    // 0       * x   + E[1][1] * y   + E[1][2] * z  + ... + E[1][n] = 0
    // 0       * x   + 0       * y   + E[2][2] * z  + ... + E[2][n] = 0
    // ...
    // 0 * x + 0 * y   + ...        + E[n-1][n-1] * x_n-1 + E[n-1][n] = 0
    
    for var in (0..n).rev() {
        let (top, bottom) = equations.split_at_mut(var);
        let row_var = &mut bottom[0];
        for t in top {
            eliminate_var(row_var, t, var)?;
        }
    }
    
    Ok(())
}


#[derive(Debug)]
struct User { lambdas_own: Vec<f32>, lambdas_def: Vec<f32> }
impl User {
    fn new(coefs_see_own: Vec<f32>, coefs_defer_feeds: Vec<f32>) -> Result<User, &'static str> {
        if coefs_see_own.len() != coefs_defer_feeds.len() {
            return Err("invalid length");
        }
        if (coefs_see_own.iter().sum::<f32>()
          + coefs_defer_feeds.iter().sum::<f32>() - 1.0f32).abs() > f32::EPSILON {
            return Err("invalid sum");
        }
        
        Ok(User {lambdas_own: coefs_see_own, lambdas_def: coefs_defer_feeds})
    }
}

struct EigenflowApp { users: Vec<User> }
impl EigenflowApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let user_a = User::new(vec![0.8, 0.0, 0.0], vec![0.0, 0.1, 0.1]).unwrap();
        let user_b = User::new(vec![0.4, 0.1, 0.0], vec![0.0, 0.0, 0.5]).unwrap();
        let user_c = User::new(vec![0.0, 0.2, 0.3], vec![0.0, 0.5, 0.0]).unwrap();
        
        EigenflowApp {users: vec![user_a, user_b, user_c]}
    }
    
    fn calculate_views(&self) -> (Vec<Vec<f32>>, bool) {
        let n = self.users.len();
        
        // return vec![vec![0.0; n]; n];
        // return self.users.iter().map(|user| user.lambdas_own.clone()).collect();
        
        let mut flow = vec![vec![0.0; n]; n];
        let mut valid = true;
        
        for poster in 0..n {
            let mut equations = vec![vec![0.0; n + 1]; n];
            for viewer in 0..n {
                equations[viewer][0..n].copy_from_slice(&self.users[viewer].lambdas_def);
                equations[viewer][viewer] -= 1.0;
                equations[viewer][n] = self.users[viewer].lambdas_own[poster];
            }
            
            valid = solve_lineq(&mut equations).is_ok() && valid;
            
            for viewer in 0..n {
                flow[poster][viewer] = -equations[viewer][n] / equations[viewer][viewer];
            }
        }
        
        for i in 0..n {
            for j in 0..i {
                let (flow_j, flow_i) = (&mut flow).split_at_mut(i);  // uses only one &mut ref
                std::mem::swap(&mut flow_i[0][j], &mut flow_j[j][i]);
            }
        }
        (flow, valid)
    }
}

impl App for EigenflowApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        let (feed, valid) = self.calculate_views();
        let left: Vec<f32> = feed.iter().map(|v| 1.0 - v.iter().sum::<f32>()).collect();
        
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.set_min_height(30.0);
                if ui.button("Quit").clicked() { frame.close(); }
                ui.separator();
                label!(to ui, if valid {"consistent"} else {"invalid lin system"});
            });
        });
        
        CentralPanel::default().frame(frame!(margin: 8.0, fill: 24 24 24)).show(ctx, |ui| {
            ScrollArea::vertical().show_rows(ui, 88.0, self.users.len(), |ui, row_range| {
                ui.vertical(|ui| {
                    for row in row_range {
                        let c = format!("User #{row} = {:?}\n\nsees {:?}, left {:.1}",
                            self.users[row], feed[row], left[row]);
                        sized_button!(to ui, height 72.0,
                          text text!(&c, color color!(255, 255, 255)),
                          fill color!(48, 48, 48));
                        ui.add_space(16.0);
                    }
                });
            });
        });
    }
}


//------------------------------------------------------------------------------

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "EigenFlow",
        native_options,
        Box::new(|cc| {
            Box::new(EigenflowApp::new(cc))
        }),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| {
                    Box::new(EigenflowApp::new(cc))
                }),
            )
            .await
            .expect("failed to start eframe");
    });
}
