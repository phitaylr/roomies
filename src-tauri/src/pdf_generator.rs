use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use chrono::Local;
use crate::solver::{SolveResult, Person};

const PAGE_WIDTH: f32 = 210.0;
const PAGE_HEIGHT: f32 = 297.0;
const MARGIN_LEFT: f32 = 20.0;
const MARGIN_RIGHT: f32 = 190.0;
const LINE_HEIGHT: f32 = 5.0;
const BOTTOM_MARGIN: f32 = 20.0;

pub fn generate_pdf(
    result: &SolveResult,
    event_name: &str,
    people: &[Person],
    output_path: &Path,
) -> Result<String, String> {
    let (doc, page1, layer1) = PdfDocument::new(
        event_name,
        Mm(PAGE_WIDTH),
        Mm(PAGE_HEIGHT),
        "Layer 1"
    );
    
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).map_err(|e| e.to_string())?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).map_err(|e| e.to_string())?;
    
    let mut current_page = page1;
    let mut current_layer = doc.get_page(current_page).get_layer(layer1);
    let mut y = PAGE_HEIGHT - 20.0;
    
    // Helper function to check if we need a new page
    let mut add_new_page = |doc: &PdfDocumentReference, y: &mut f32| -> (PdfPageIndex, PdfLayerIndex) {
        let (page, layer) = doc.add_page(Mm(PAGE_WIDTH), Mm(PAGE_HEIGHT), "Layer 1");
        *y = PAGE_HEIGHT - 20.0;
        (page, layer)
    };
    
    // Title
    current_layer.use_text(event_name, 20.0, Mm(MARGIN_LEFT), Mm(y), &font_bold);
    y -= LINE_HEIGHT * 2.5;
    
    // Date
    let date = Local::now().format("%B %d, %Y").to_string();
    current_layer.use_text(&format!("Generated: {}", date), 10.0, Mm(MARGIN_LEFT), Mm(y), &font);
    y -= LINE_HEIGHT * 3.0;
    
    // Summary section
    current_layer.use_text("Summary", 14.0, Mm(MARGIN_LEFT), Mm(y), &font_bold);
    y -= LINE_HEIGHT * 1.5;
    
    let summary_items = vec![
        format!("Total Rooms: {}", result.total_rooms),
        format!("Choice Satisfaction Score: {}", result.choice_score),
        format!("Room Balance (Imbalance): {}", result.imbalance),
        format!("People without choices: {}", result.without_choices),
    ];
    
    for item in summary_items {
        current_layer.use_text(&item, 10.0, Mm(MARGIN_LEFT + 5.0), Mm(y), &font);
        y -= LINE_HEIGHT;
    }
    y -= LINE_HEIGHT;
    
    // Warnings section
    if result.without_choices > 0 {
        if y < BOTTOM_MARGIN + 30.0 {
            let (page, layer) = add_new_page(&doc, &mut y);
            current_page = page;
            current_layer = doc.get_page(current_page).get_layer(layer);
        }
        
        current_layer.use_text("Warnings", 12.0, Mm(MARGIN_LEFT), Mm(y), &font_bold);
        y -= LINE_HEIGHT * 1.5;
        
        for (_category, rooms) in &result.rooms_by_category {
            for room in rooms {
                for person_name in room {
                    if let Some(person) = people.iter().find(|p| &p.name == person_name) {
                        if !person.choices.is_empty() {
                            let has_choice = person.choices.iter().any(|c| room.contains(c));
                            if !has_choice {
                                if y < BOTTOM_MARGIN + 10.0 {
                                    let (page, layer) = add_new_page(&doc, &mut y);
                                    current_page = page;
                                    current_layer = doc.get_page(current_page).get_layer(layer);
                                }
                                
                                let text = format!("• {} has none of their choices in their room", person.name);
                                current_layer.use_text(&text, 9.0, Mm(MARGIN_LEFT + 5.0), Mm(y), &font);
                                y -= LINE_HEIGHT * 0.9;
                            }
                        }
                    }
                }
            }
        }
        y -= LINE_HEIGHT;
    }
    
    // Room assignments section
   // Room assignments section
if y < BOTTOM_MARGIN + 40.0 {
    let (page, layer) = add_new_page(&doc, &mut y);
    current_page = page;
    current_layer = doc.get_page(current_page).get_layer(layer);
}

current_layer.use_text("Room Assignments", 14.0, Mm(MARGIN_LEFT), Mm(y), &font_bold);
y -= LINE_HEIGHT * 2.0;

let mut room_num = 1;
let column_width = 85.0;
let left_column_x = MARGIN_LEFT;
let right_column_x = MARGIN_LEFT + column_width + 10.0;

for (category, rooms) in &result.rooms_by_category {
    // Category header spans both columns
    if y < BOTTOM_MARGIN + 20.0 {
        let (page, layer) = add_new_page(&doc, &mut y);
        current_page = page;
        current_layer = doc.get_page(current_page).get_layer(layer);
    }
    
    current_layer.use_text(&format!("{} Rooms:", category), 12.0, Mm(left_column_x), Mm(y), &font_bold);
    y -= LINE_HEIGHT * 1.5;
    
    let mut column = 0; // 0 = left, 1 = right
    let mut left_column_y = y;
    let mut right_column_y = y;
    
    for room in rooms {
        let room_height = (room.len() as f32 * LINE_HEIGHT * 0.7) + LINE_HEIGHT * 1.5;
        
        // Determine which column to use
        let (x_pos, column_y) = if column == 0 {
            (left_column_x + 5.0, &mut left_column_y)
        } else {
            (right_column_x, &mut right_column_y)
        };
        
        // Check if we need a new page
        if *column_y < BOTTOM_MARGIN + room_height {
            if column == 0 {
                // Try right column first
                column = 1;
                if right_column_y < BOTTOM_MARGIN + room_height {
                    // Both columns full, new page
                    let (page, layer) = add_new_page(&doc, &mut y);
                    current_page = page;
                    current_layer = doc.get_page(current_page).get_layer(layer);
                    left_column_y = y;
                    right_column_y = y;
                    column = 0;
                }
                continue; // Retry with new column/page
            } else {
                // Right column full, new page
                let (page, layer) = add_new_page(&doc, &mut y);
                current_page = page;
                current_layer = doc.get_page(current_page).get_layer(layer);
                left_column_y = y;
                right_column_y = y;
                column = 0;
                continue; // Retry with new page
            }
        }
        
        let x_pos = if column == 0 { left_column_x + 5.0 } else { right_column_x };
        let column_y = if column == 0 { &mut left_column_y } else { &mut right_column_y };
        
        // Room header
        current_layer.use_text(
            &format!("Room {} ({} people)", room_num, room.len()), 
            9.0, Mm(x_pos), Mm(*column_y), &font_bold
        );
        *column_y -= LINE_HEIGHT * 0.9;
        
        // Room members
        for member in room {
            let display_name = if member.len() > 22 {
                format!("• {}...", &member[..19])
            } else {
                format!("• {}", member)
            };
            current_layer.use_text(&display_name, 8.0, Mm(x_pos + 2.0), Mm(*column_y), &font);
            *column_y -= LINE_HEIGHT * 0.7;
        }
        
        room_num += 1;
        *column_y -= LINE_HEIGHT * 0.5; // Space between rooms
        
        // Alternate columns
        column = 1 - column;
    }
    
    // After category, reset to single column on new section
    y = left_column_y.min(right_column_y) - LINE_HEIGHT;
}
    
    
    // Room details section
    let (page, layer) = add_new_page(&doc, &mut y);
    current_page = page;
    current_layer = doc.get_page(current_page).get_layer(layer);
    
    current_layer.use_text("Room Details & Relationships", 14.0, Mm(MARGIN_LEFT), Mm(y), &font_bold);
    y -= LINE_HEIGHT * 2.0;
    
    room_num = 1;
    for (_category, rooms) in &result.rooms_by_category {
        for room in rooms {
            let estimated_height = room.len() as f32 * LINE_HEIGHT * 0.9 + LINE_HEIGHT * 2.0;
            if y < BOTTOM_MARGIN + estimated_height {
                let (page, layer) = add_new_page(&doc, &mut y);
                current_page = page;
                current_layer = doc.get_page(current_page).get_layer(layer);
            }
            
            current_layer.use_text(&format!("Room {}:", room_num), 10.0, Mm(MARGIN_LEFT), Mm(y), &font_bold);
            y -= LINE_HEIGHT;
            
            for person_name in room {
                if let Some(person) = people.iter().find(|p| &p.name == person_name) {
                    let mut parts = Vec::new();
                    
                    // Who they chose in this room
                    let chose: Vec<&str> = person.choices.iter()
                        .filter(|c| room.contains(c))
                        .map(|s| s.as_str())
                        .collect();
                    if !chose.is_empty() {
                        parts.push(format!("chose {}", chose.join(", ")));
                    }
                    
                    // Who chose them
                    let mut chosen_by = Vec::new();
                    for other_name in room {
                        if other_name != person_name {
                            if let Some(other) = people.iter().find(|p| &p.name == other_name) {
                                if other.choices.contains(person_name) {
                                    chosen_by.push(other_name.as_str());
                                }
                            }
                        }
                    }
                    if !chosen_by.is_empty() {
                        parts.push(format!("chosen by {}", chosen_by.join(", ")));
                    }
                    
                    // Avoids
                    if !person.avoids.is_empty() {
                        parts.push(format!("avoids {}", person.avoids.join(", ")));
                    }
                    
                    let text = if parts.is_empty() {
                        format!("  • {}", person.name)
                    } else {
                        format!("  • {}: {}", person.name, parts.join("; "))
                    };
                    
                    // Handle long text wrapping
                    if text.len() > 90 {
                        // Split into multiple lines if too long
                        let line1 = &text[..90.min(text.len())];
                        current_layer.use_text(line1, 8.0, Mm(MARGIN_LEFT + 5.0), Mm(y), &font);
                        y -= LINE_HEIGHT * 0.8;
                        
                        if text.len() > 90 {
                            let line2 = &text[90..];
                            current_layer.use_text(&format!("    {}", line2), 8.0, Mm(MARGIN_LEFT + 5.0), Mm(y), &font);
                            y -= LINE_HEIGHT * 0.8;
                        }
                    } else {
                        current_layer.use_text(&text, 8.0, Mm(MARGIN_LEFT + 5.0), Mm(y), &font);
                        y -= LINE_HEIGHT * 0.8;
                    }
                }
            }
            room_num += 1;
            y -= LINE_HEIGHT * 0.8;
        }
    }
    
    // Save
    doc.save(&mut BufWriter::new(File::create(output_path).map_err(|e| e.to_string())?))
        .map_err(|e| e.to_string())?;
    
    Ok(output_path.to_string_lossy().to_string())
}