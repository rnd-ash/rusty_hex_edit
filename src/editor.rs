use std::{fs::File, io::Read};

use eframe::{egui::{self, RichText, Label, Layout, TextEdit, Direction, style::Spacing}, emath::{Vec2, Rect}, epaint::Color32};
use egui_extras::{TableBuilder, Size, Strip, StripBuilder};
use nfd::Response;
use wgpu::Color;
use std::fmt::Write;

pub enum CellSize {
    Bit,
    Byte,
    Short,
    Int,
    Long
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Repr {
    Ascii,
    Mapbars
}


pub struct Editor{
    open_bytes: Vec<u8>,
    edit_bytes: Vec<u8>,
    cell_size: CellSize,
    bytes_per_row: usize,
    repr: Repr
}

impl Editor {
    pub fn new() -> Self {
        Self{
            open_bytes: Vec::new(),
            edit_bytes: Vec::new(),
            cell_size: CellSize::Byte,
            bytes_per_row: 16,
            repr: Repr::Ascii
        }
    }
}

impl eframe::App for Editor {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {


        egui::CentralPanel::default().show(ctx, |ui| {
            let dark_mode = ui.visuals().dark_mode;
            let faded_color = ui.visuals().window_fill();
            let faded_color = |color: Color32| -> Color32 {
                use egui::Rgba;
                let t = if dark_mode { 0.95 } else { 0.8 };
                egui::lerp(Rgba::from(color)..=Rgba::from(faded_color), t).into()
            };
            egui::menu::bar(ui, |bar_ui| {
                bar_ui.menu_button("File", |file_menu| {
                    if file_menu.button("About").clicked() {

                    }
                    if file_menu.button("Open").clicked() {
                        if let Ok(resp) = nfd::open_dialog(None, None, nfd::DialogType::SingleFile) {
                            if let Response::Okay(path) = resp {
                                let mut f = File::open(path).unwrap();
                                self.open_bytes.clear();
                                let _ = f.read_to_end(&mut self.open_bytes);
                                self.edit_bytes = self.open_bytes.clone();
                            }
                        }
                        file_menu.close_menu();
                    }
                    if file_menu.button("Save").clicked() {

                    }
                });
            });
            ui.add(egui::Separator::default());
            
            if self.open_bytes.is_empty() {
                ui.heading("No file open");
            } else {
                egui::containers::Window::new("Hex view").show(ui.ctx(), |window_view|{
                    egui::menu::bar(window_view, |bar_ui| {
                        bar_ui.menu_button("Representation", |repr_menu| {
                            if repr_menu.radio(self.repr == Repr::Ascii, "ASCII").clicked() {
                                self.repr = Repr::Ascii;
                            }
                            if repr_menu.radio(self.repr == Repr::Mapbars, "Map bars").clicked() {
                                self.repr = Repr::Mapbars;
                            }
                        });
                    });
                    window_view.add(egui::Separator::default());


                    // Show the contents of the file
                    let total_rows = self.open_bytes.len() / self.bytes_per_row;
                    let row_height = ui.text_style_height(&egui::TextStyle::Body);
                    
                    let mut table_builder = TableBuilder::new(window_view)
                        .striped(true)
                        .cell_layout(Layout::left_to_right().with_cross_align(egui::Align::Center));

                    table_builder = table_builder.column(Size::initial(60.0).at_least(60.0));
                    // For each offset
                    for _ in 0..self.bytes_per_row {
                        table_builder = table_builder.column(Size::initial(40.0).at_least(60.0));
                    }
                    table_builder = table_builder.column(Size::initial(self.bytes_per_row as f32 * row_height).at_least(self.bytes_per_row as f32 * row_height));


                    let table = table_builder.header(15.0, | mut header | {
                        header.col(|ui| { ui.label(RichText::new("Offset")); });
                        
                        for x in 0..self.bytes_per_row {
                            header.col(|ui| { ui.label(RichText::new(format!("+0x{:02X}", x))); });
                        }
                        header.col(|ui| { { ui.label(RichText::new("Representation")); } });
                    })
                    .body(|mut body| {
                        let mut row_count = self.edit_bytes.len() / self.bytes_per_row;
                        if self.edit_bytes.len() % self.bytes_per_row != 0 {
                            row_count += 1;
                        }
                        body.rows(18.0, row_count, |row_id, mut row| {
                            row.col(|ui| { ui.label(format!("0x{:06X}", row_id*self.bytes_per_row )); }); // Offset
                                for i in 0..self.bytes_per_row {
                                    if let Some(byte) = self.edit_bytes.get_mut((row_id*self.bytes_per_row) + i) {
                                        row.col(|ui| {
                                            let orig = self.open_bytes[(row_id*self.bytes_per_row) + i];
                                            let mut s = format!("{:02X}", byte);
                                            let c = s.clone();

                                            let mut edit = TextEdit::singleline(&mut s);
                                            if orig != *byte {
                                                edit = edit.text_color(Color32::from_rgb(255, 0, 0))
                                            }

                                            ui.add(edit);
                                            if s != c {
                                                // User changed the value in a cell
                                                if let Ok(new_byte) = u8::from_str_radix(&s, 16) {
                                                    *byte = new_byte;
                                                }
                                            }
                                        });
                                    } else { row.col(|_| {}); }
                                }
                            row.col(|ui| {
                                ui.style_mut().spacing.item_spacing.x = 0.0;
                                let mut strip_builder = StripBuilder::new(ui).cell_layout(Layout::centered_and_justified(Direction::BottomUp));
                                for i in 0..self.bytes_per_row {
                                    strip_builder = strip_builder.size(Size::relative(1.0 / self.bytes_per_row as f32));
                                }
                                strip_builder.horizontal(|mut strip| {
                                    for i in 0..self.bytes_per_row {
                                        strip.cell(|cell| {
                                            if let Some(byte) = self.edit_bytes.get((row_id*self.bytes_per_row) + i) {
                                                // If map representation

                                                match self.repr {
                                                    Repr::Ascii => {
                                                        // If Ascii representation
                                                        if byte.is_ascii_graphic() {
                                                            cell.label(format!("{}", String::from_utf8_lossy(&[*byte])));
                                                        } else {
                                                            cell.label(".");
                                                        }
                                                    },
                                                    Repr::Mapbars => {
                                                        let mut rect = cell.available_rect_before_wrap();
                                                        let total_height = rect.height();

                                                        let bottom = rect.bottom();
                                                        let top = rect.top();
                                                        let dy = top - bottom;
                                                        *rect.top_mut() = bottom + ( (*byte as f32 / 255.0) * dy );
                                                        cell.painter().rect_filled(rect, 0.0, faded_color(Color32::BLUE));
                                                    },
                                                }
                                            }
                                        });
                                    }
                                });
                            });
                        });
                    });
                });
            }
        });
    }
}