#![allow(dead_code)]
#![allow(static_mut_refs)] // Yeah, this is probably fine

// Bugs:
// When a block structure is added to a another block structure, subsequent blocks of the moved blocks don't get tagged with 'recently_moved'
// There is a line where the code keeps on erroring ->

use core::panic;
use fontdue::Font;
use mirl::graphics::rgb_to_u32;
use std::collections::HashMap;

use crate::custom::BlockInput;
use crate::custom::WorkSpace;
use crate::logic::Physics;
use crate::platform::shared::FileSystem;
use crate::platform::KeyCode;
use crate::render::RenderSettings;

use crate::custom::Block;
use crate::custom::Camera;
use crate::custom::ID;

// if false {
//     let found = get_pixel(buffer, width, height, x, y);
//     if found != 0 {
//         return;
//     }
// }
// #[inline]
// fn get_pixel(
//     buffer: &Vec<u32>,
//     width: &isize,
//     height: &isize,
//     x: isize,
//     y: isize,
// ) -> u32 {
//     if x < 0 || y < 0 {
//         return 0;
//     }
//     if x >= *width || y >= *height {
//         return 0;
//     }
//     let index = y * width + x;
//     return buffer[index as usize];
// }

#[inline]
fn render_block<R: RenderSettings, L: Physics>(
    block: &Block,
    camera: &Camera,
    buffer: *mut u32,
    block_colors: &Vec<u32>,
    width: usize,
    height: usize,
    font: &Font,
    render_setting: &R,
    physics: &L,
) {
    render_setting.draw_block(
        block,
        block.x.get() as isize,
        block.y.get() as isize,
        camera,
        buffer,
        block_colors[block.block_color_id],
        width,
        height,
        font,
        physics,
    );
}
#[inline]
fn get_top_most_block_id_or_self(blocks: &Vec<Block>, index: usize) -> ID {
    // Is this broken???????
    // PROBABLY NOT BUT SOMEONE WHO CALLS THIS FUNCTION DEFENITELY IS
    let block = &blocks[index];
    if block.connected_top.get().is_some() {
        return block.connected_top.get().unwrap();
    }
    return block.id;
}

fn handle_connection_and_render_ghost_block<R: RenderSettings, L: Physics>(
    buffer: *mut u32,
    width: &usize,
    height: &usize,
    camera: &Camera,
    blocks: &Vec<Block>,
    font: &Font,
    selected: &Option<usize>,
    snap_distance: f32,
    render_settings: &R,
    logic: &L,
    block_colors: &Vec<u32>,
) {
    if selected.is_some() {
        // Connect to block above
        let possible = logic.get_block_in_distance(
            &blocks,
            blocks[selected.unwrap()].x.get() as f32,
            blocks[selected.unwrap()].y.get() as f32,
            snap_distance,
            *selected,
            true,
        );
        let block = &blocks[selected.unwrap()];
        block.possible_connection_above.set(None);

        if possible.is_some() {
            let above_block = &blocks[possible.unwrap()];
            if is_there_a_loop_in_block_connections_for_block_internal(
                &blocks,
                get_top_most_block_id_or_self(
                    blocks,
                    index_by_block_id(above_block.id, blocks).unwrap(),
                ),
                &mut Vec::from([block.id]),
            ) {
                //println!("Loop avoided");
                return;
            } else {
                // Save block, only if it is not in a loop
                block.possible_connection_above.set(Some(above_block.id));
            }

            if !logic.is_block_visible_on_screen(
                &above_block,
                camera,
                &(*width as isize),
                &(*height as isize),
            ) {
                return;
            }
            render_settings.draw_block(
                &block,
                above_block.x.get() as isize,
                above_block.y.get() as isize + block.height.get() as isize,
                camera,
                buffer,
                render_settings.desaturate(
                    render_settings.adjust_brightness(
                        block_colors[block.block_color_id],
                        -5,
                    ),
                    0.91,
                ),
                *width,
                *height,
                font,
                logic,
            );
        }
    }
}

// Misc
fn handle_and_render_action_blocks_on_screen<R: RenderSettings, L: Physics>(
    buffer: *mut u32,
    width: &usize,
    height: &usize,
    camera: &Camera,
    blocks: &mut Vec<Block>,
    block_colors: &Vec<u32>,
    font: &Font,
    render_settings: &R,
    logic: &L,
) {
    let now_width = *width as isize;
    let now_height = *height as isize;

    let block_ids: Vec<usize> = (0..blocks.len()).collect();

    // Reverse block order in order for overdraw to to its job in our favor
    for id in block_ids {
        if blocks[id].recently_moved.get() {
            move_block_to_connected(blocks, &Some(id));
        }
        let block = &blocks[id];

        if !logic.is_block_visible_on_screen(
            block,
            camera,
            &now_width,
            &now_height,
        ) {
            continue;
        }
        render_block(
            block,
            camera,
            buffer,
            block_colors,
            *width,
            *height,
            font,
            render_settings,
            logic,
        );
    }
}
fn handle_and_render_inline_blocks_on_screen<R: RenderSettings, L: Physics>(
    buffer: *mut u32,
    width: &usize,
    height: &usize,
    camera: &Camera,
    blocks: &mut Vec<Block>,
    block_colors: &Vec<u32>,
    font: &Font,
    render_settings: &R,
    logic: &L,
) {
    let now_width = *width as isize;
    let now_height = *height as isize;

    let block_ids: Vec<usize> = (0..blocks.len()).collect();

    // Reverse block order in order for overdraw to to its job in our favor
    for id in block_ids {
        if blocks[id].recently_moved.get() {
            move_block_to_connected(blocks, &Some(id));
        }
        let block = &blocks[id];

        if !logic.is_block_visible_on_screen(
            block,
            camera,
            &now_width,
            &now_height,
        ) {
            continue;
        }
        render_block(
            block,
            camera,
            buffer,
            block_colors,
            *width,
            *height,
            font,
            render_settings,
            logic,
        );
    }
}

#[inline]
fn subtract_tuple(one: (f32, f32), two: (f32, f32)) -> (f32, f32) {
    (one.0 - two.0, one.1 - two.1)
}
#[inline]
fn reorder_element<T>(vec: &mut Vec<T>, from: usize, to: usize) {
    if from != to && from < vec.len() && to < vec.len() {
        let item = vec.remove(from);
        vec.insert(to, item)
    }
}

// Block stuff
fn get_block_id_above<L: Physics>(
    blocks: &Vec<Block>,
    pos_x: f32,
    pos_y: f32,
    logic: &L,
) -> Option<usize> {
    for block_id in 0..blocks.len() {
        let block = &blocks[block_id];
        if logic.is_point_in_requctangle(
            pos_x,
            pos_y,
            block.x.get() as f32,
            block.y.get() as f32 + block.height.get(),
            block.width.get() as f32,
            block.height.get() as f32,
        ) {
            return Some(block_id);
        }
    }
    return None;
}

#[inline]
fn get_block_id_under_point<L: Physics>(
    blocks: &Vec<Block>,
    pos_x: f32,
    pos_y: f32,
    logic: &L,
) -> Option<usize> {
    for block_id in 0..blocks.len() {
        let block = &blocks[block_id];
        if logic.is_point_in_requctangle(
            pos_x,
            pos_y,
            block.x.get() as f32,
            block.y.get() as f32,
            block.width.get() as f32,
            block.height.get() as f32,
        ) {
            return Some(block_id);
        }
    }
    return None;
}

fn add_item_to_max_sized_list(list: &mut Vec<u64>, max_size: usize, item: u64) {
    list.push(item);
    if list.len() < max_size {
        return;
    }
    let to_remove = list.len() - max_size;
    for _ in 0..to_remove {
        list.remove(0);
    }
}

#[inline]
fn index_by_block_id(id: ID, blocks: &Vec<Block>) -> Option<usize> {
    for block_id in 0..blocks.len() {
        if blocks[block_id].id == id {
            return Some(block_id);
        }
    }
    return None;
}

fn is_there_a_loop_in_block_connections_for_block(
    blocks: &Vec<Block>,
    block_index: ID,
) -> bool {
    let mut already_checked = Vec::new();
    return is_there_a_loop_in_block_connections_for_block_internal(
        blocks,
        block_index,
        &mut already_checked,
    );
}
fn is_there_a_loop_in_block_connections_for_block_internal(
    blocks: &Vec<Block>,
    block_id: ID,
    already_checked: &mut Vec<ID>,
) -> bool {
    // I don't think this function works quite right
    if already_checked.contains(&block_id) {
        return true;
    }
    already_checked.push(block_id);
    let block = &blocks[index_by_block_id(block_id, blocks).unwrap()];
    if block.connected_below.get().is_some() {
        return is_there_a_loop_in_block_connections_for_block_internal(
            blocks,
            block.connected_below.get().unwrap(),
            already_checked,
        );
    }
    // if block.connected_below.get().is_some() {
    //     return is_there_a_loop_in_block_connections_for_block(
    //         blocks,
    //         block.connected_below.get().unwrap(),
    //         already_checked,
    //     );
    // }
    return false;
}
#[inline]
fn custom_join(vector: Vec<ID>, separator: &str) -> String {
    let mut out = "".to_string();
    let length = vector.len();
    for idx in 0..length {
        let item = vector[idx].to_string();
        out = out + &item;
        if idx != length - 1 {
            out = out + separator;
        }
    }
    return out;
}

#[inline]
fn get_ids_connected_to_block(
    top_most_block_id: ID,
    blocks: &Vec<Block>,
    found: &mut Vec<ID>,
) -> Vec<ID> {
    // FIX THIS FUNCTION FOR GOD SAKE WHY ARE THERE MUTLIPLE IDS??????????????????
    let block = &blocks[index_by_block_id(top_most_block_id, &blocks).unwrap()];

    if found.contains(&block.id) {
        println!("{} -> {}", block.id, block.connected_below.get().unwrap());
        panic!(
            "Infinite loop found with these ids {} with current {} (len({}))",
            custom_join(found.to_vec(), ", "),
            top_most_block_id,
            found.len()
        )
    }
    found.push(top_most_block_id);
    if block.connected_below.get().is_some() {
        let sub = get_ids_connected_to_block(
            block.connected_below.get().unwrap(),
            blocks,
            found,
        );

        found.extend(sub)
    }

    // Filter out multiple of an id -> WE SHOULD NOT NEED TO DO THIS
    let mut seen = std::collections::HashSet::new();

    found.retain(|x| seen.insert(*x));

    return found.to_vec();
}
#[inline]
fn index_by_block_ids(
    blocks: &Vec<Block>,
    ids: &Vec<ID>,
) -> Vec<Option<usize>> {
    let mut return_list = Vec::new();
    for id in ids {
        return_list.push(index_by_block_id(*id, blocks))
    }
    return return_list;
}
fn get_total_height_of_blocks(
    blocks: &Vec<Block>,
    indexes: Vec<Option<usize>>,
) -> f32 {
    // For optimization, remove the Option<>
    let mut height: f32 = 0.0;
    for idx in indexes {
        if idx.is_none() {
            continue;
        }
        let block = &blocks[idx.unwrap()];
        height += block.height.get();
    }
    return height;
}

fn get_usize_out_of_option_list(list: Vec<Option<usize>>) -> Vec<usize> {
    let mut new = Vec::new();
    for l in list {
        new.push(l.unwrap())
    }
    return new;
}

fn move_block_to_connected(blocks: &mut Vec<Block>, index: &Option<usize>) {
    if index.is_none() {
        return;
    }
    let block = &blocks[index.unwrap()];
    let top_block_id = block.connected_top.get();
    if top_block_id.is_none() {
        return;
    }
    let top_block =
        &blocks[index_by_block_id(top_block_id.unwrap(), blocks).unwrap()];
    let block_query = get_ids_connected_to_block(
        top_block_id.unwrap(),
        blocks,
        &mut Vec::new(),
    );
    // THIS ERRORS vvv
    let own_block_id_index = block_query
        .iter()
        .position(|&id| id == block.id)
        .expect("FIX ME GOD DAMN IT");

    let blocks_to_offset_with =
        &block_query[0..own_block_id_index + 1].to_vec();

    //println!(">{:?}", block_query);

    let total_offset = get_total_height_of_blocks(
        blocks,
        index_by_block_ids(blocks, blocks_to_offset_with),
    );

    block.x.set(top_block.x.get());
    block.y.set(
        total_offset as u16 + top_block.y.get() - block.height.get() as u16,
    );
    block.recently_moved.set(false);
    // blocks[above_block.unwrap()].x.set(blocks[selected.unwrap()].x.get());
    // blocks[above_block.unwrap()].y.set(
    //     blocks[selected.unwrap()].y.get()
    //         + blocks[selected.unwrap()].height.get() as u16,
    // );
}
#[inline]
fn get_difference_of_values_in_percent(value1: f64, value2: f64) -> f64 {
    if value1 == 0.0 {
        return 100.0; // Avoid division by zero, assuming 100% difference
    }
    ((value2 - value1) / value1).abs() * 100.0
}
fn handle_mouse<F: platform::shared::Framework>(
    camera: &mut Camera,
    framework: &F,
    scroll_multiplier: f32,
) {
    // Mouse wheel movement
    let mouse_wheel_temp = framework.get_mouse_scroll();
    if mouse_wheel_temp.is_some() {
        let extra_mul;
        if framework.is_key_down(platform::KeyCode::RightShift) {
            extra_mul = 10.0;
        } else {
            extra_mul = 1.0;
        }
        if framework.is_key_down(platform::KeyCode::LeftControl) {
            camera.z -= mouse_wheel_temp.unwrap().1 * extra_mul;
        } else {
            if framework.is_key_down(platform::KeyCode::LeftShift) {
                camera.y -= (mouse_wheel_temp.unwrap().0
                    * scroll_multiplier
                    * extra_mul) as isize;
                camera.x -= (mouse_wheel_temp.unwrap().1
                    * scroll_multiplier
                    * extra_mul) as isize;
            } else {
                camera.x -= (mouse_wheel_temp.unwrap().0
                    * scroll_multiplier
                    * extra_mul) as isize;
                camera.y -= (mouse_wheel_temp.unwrap().1
                    * scroll_multiplier
                    * extra_mul) as isize;
            }
        }
    }
}
fn reorder_blocks(
    mouse_down: bool,
    blocks: &mut Vec<Block>,
    camera: &mut Camera,
    mouse_delta: (f32, f32),
    selected: &mut Option<usize>,
) {
    if mouse_down {
        if let Some(mut idx) = selected {
            reorder_element(blocks, idx, 0);
            idx = 0;
            *selected = Some(0);
            blocks[idx]
                .x
                .set((blocks[idx].x.get() as f32 + mouse_delta.0) as u16);
            blocks[idx]
                .y
                .set((blocks[idx].y.get() as f32 + mouse_delta.1) as u16);
        } else {
            camera.x -= mouse_delta.0 as isize;
            camera.y -= mouse_delta.1 as isize;
        }
    }
}
fn handle_or_get_selected<
    //F: platform::shared::Framework,
    L: Physics,
>(
    mouse_down: bool,
    last_mouse_down: bool,
    blocks: &mut Vec<Block>,
    camera: &mut Camera,
    mouse_pos: (f32, f32),
    // mouse_delta: (f32, f32),
    // framework: &F,
    // scroll_multiplier: f32,
    selected: Option<usize>,
    logic: &L,
    selected_type_is_action: bool,
) -> Option<usize> {
    // There are too many problems with dealing with null when the mouse is outside the window, so instead we just check if the mouse is with in the window :)
    let mut selected = selected;
    let _selected_type_is_action = selected_type_is_action;
    if mouse_down {
        if !last_mouse_down {
            selected = get_block_id_under_point(
                &blocks,
                mouse_pos.0 + camera.x as f32,
                mouse_pos.1 + camera.y as f32,
                logic,
            );
        }
        if selected.is_some() {
            let list = get_ids_connected_to_block(
                get_top_most_block_id_or_self(&blocks, selected.unwrap()),
                &blocks,
                &mut Vec::new(),
            );
            // When this block is selected, disconnect it from the blocks above
            blocks[selected.unwrap()].connected_top.set(None);

            // find position of element with value blocks[selected.unwrap()].id
            let split_point = list
                .iter()
                .position(|block| *block == blocks[selected.unwrap()].id);

            // "Split point" -> below gets moved, above stays put, split point is selected block
            if split_point.is_none() {
                panic!("The block thought it was hanging from a block structure that isn't actually connected to this block anymore (Currently happening when replacing a block I think)")
            }

            if split_point.unwrap() > 0 {
                // Remove the connected tag from the block this block is connected to (But only if that block above exists!)
                let block_above = &blocks[index_by_block_id(
                    list[split_point.unwrap() - 1],
                    &blocks,
                )
                .unwrap()];
                block_above.connected_below.set(None);
            }

            // Tell blocks below that they have been moved
            for id in list[split_point.unwrap()..].iter() {
                blocks[index_by_block_id(*id, &blocks).unwrap()]
                    .recently_moved
                    .set(true);
            }
        }
    } else {
        // Connect block previously selected if possible
        if selected.is_some() {
            let block = &blocks[selected.unwrap()];
            if block.possible_connection_above.get().is_some() {
                let connection_id_above =
                    block.possible_connection_above.get().unwrap();

                if is_there_a_loop_in_block_connections_for_block_internal(
                    &blocks,
                    get_top_most_block_id_or_self(
                        &blocks,
                        index_by_block_id(
                            block.possible_connection_above.get().unwrap(),
                            &blocks,
                        )
                        .unwrap(),
                    ),
                    &mut Vec::from([blocks[selected.unwrap()].id]),
                ) {
                    panic!("Loop detected in block structure");
                }
                // Get above block
                let block_above_index =
                    index_by_block_id(connection_id_above, &blocks).unwrap();
                let above_block = &blocks[block_above_index];
                // Tell current block to connect above
                block.connected_top.set(Some(get_top_most_block_id_or_self(
                    &blocks,
                    block_above_index,
                )));
                // WE CURRENTLY JUST REPLACE THE CURRENT BLOCK-> THESE BLOCKS SHOULD BE APPENDED TO THE CURRENT BLOCKS AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAHHHHHHHHHHH
                // Dear past me; EXPLAIN THE CONCEPT, NOT THE EXECUTION, DUMBASS
                above_block.connected_below.set(Some(block.id));
            }

            // Set current possible above block to none
            block.possible_connection_above.set(None);
            block.recently_moved.set(true);
        }
        selected = None;
    }

    return selected;
}
use serde_json;
pub fn parse_translations(
    csv_data: &str,
    lang: &str,
) -> Result<Option<(Vec<String>, Vec<String>)>, Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true) // Add this line to handle uneven rows
        .from_reader(csv_data.as_bytes());

    let headers = rdr.headers()?.clone();
    let trimmed_lang = lang.trim();
    let lang_index = headers.iter().position(|h| h.trim() == trimmed_lang);
    let key_index = headers.iter().position(|h| h.trim() == "translation_key");

    if let (Some(k_idx), Some(l_idx)) = (key_index, lang_index) {
        let mut keys = Vec::new();
        let mut values = Vec::new();

        for result in rdr.records() {
            let record = result?;
            keys.push(record.get(k_idx).unwrap_or("").trim().to_string());
            values.push(record.get(l_idx).unwrap_or("").trim().to_string());
        }

        Ok(Some((keys, values)))
    } else {
        Ok(None)
    }
}
use csv::ReaderBuilder;
use std::error::Error;
pub fn get_length_of_text_in_font(text: &str, font: &Font) -> f32 {
    let mut length = 0.0;
    for ch in text.chars() {
        let (metrics, _) = font.rasterize(ch, 20.0);
        length += metrics.advance_width;
    }
    return length;
}

fn load_blocks<F: FileSystem, S: RenderSettings, L: Physics>(
    file_system: &F,
    block_output_types: &mut Vec<String>,
    block_output_colors: &mut Vec<u32>,
    translation: &mut HashMap<String, String>,
    font: &Font,
    workspace: &mut WorkSpace<S, L>,
) -> (Vec<Block>, Vec<Block>) {
    let path =
        r"C:\personal\games\minecraft\Automated\generation_lib\procedures";
    let mut action_blocks = Vec::new();
    let mut inline_blocks = Vec::new();

    let all_plugin_folders = file_system.get_folders_in_folder(path);
    println!("{:?}", all_plugin_folders);
    for plugin_folder in all_plugin_folders {
        let plugin_path = file_system.join(path, &plugin_folder);
        let setting_file = file_system.join(&plugin_path, "settings.json");
        let translation_file =
            file_system.join(&plugin_path, "translation.csv");

        let translation_data =
            file_system.get_file_contents(&translation_file).unwrap();
        let extracted =
            parse_translations(&translation_data.as_string().unwrap(), "en")
                .unwrap()
                .expect("Could not find language");

        for i in 0..extracted.0.len() {
            translation.insert(extracted.0[i].clone(), extracted.1[i].clone());
        }
        // translation_keys.extend(extracted.0);
        // translation_values.extend(extracted.1);

        let settings_json: serde_json::Value = serde_json::from_str(
            &file_system
                .get_file_contents(&setting_file)
                .unwrap()
                .as_string()
                .unwrap(),
        )
        .expect("Unable to load json settings file");
        let pre_blocks = settings_json
            .get("blocks")
            .ok_or("Error")
            .expect("Error unwrapping");
        for block in
            pre_blocks.as_array().ok_or("Error").expect("Error unwrapping")
        {
            println!(
                "{}",
                block.get("name").ok_or("Error").expect("Error unwrapping")
            );
            let block_type =
                block.get("type").ok_or("Error").expect("Error unwrapping");
            if block_type == "action" || block_type == "inline" {
                let internal_name: String = block
                    .get("name")
                    .expect("Missing name")
                    .as_str()
                    .expect("Error unwrapping")
                    .to_string();

                let output = block
                    .get("output")
                    .expect(&format!("Missing output for {}", internal_name))
                    .as_str()
                    .expect("Error unwrapping")
                    .to_string();

                if !block_output_types.contains(&output) {
                    block_output_types.push(output.clone());
                    block_output_colors.push(generate_random_color());
                }

                let name = translation.get(&internal_name).expect(&format!(
                    "Translation key '{}' not found",
                    internal_name
                ));

                let _empty = serde_json::Value::Array(Vec::new());
                let pre_inputs = block
                    .get("inputs")
                    .unwrap_or_else(|| &_empty)
                    .as_array()
                    .expect("Error unwrapping");

                let mut inputs = Vec::new();
                for pre_input in pre_inputs {
                    let temp = pre_input.as_object().unwrap();
                    let key_and_return_temp;
                    let in_case_of_literal_keys_and_return_values =
                        temp.get("expected");

                    let mut expected_literal_allowed = Vec::new();
                    let mut expected_literal_return = Vec::new();

                    if in_case_of_literal_keys_and_return_values.is_some() {
                        key_and_return_temp =
                            in_case_of_literal_keys_and_return_values
                                .expect("THIS SHOULD LITERALLY BE IMPOSSIBLE")
                                .as_object();
                        //println!("{:?}", key_and_return_temp);

                        let keys = key_and_return_temp.expect("Expected key 'expected' to be a dict, are you sure you didn't accidentally make it a list?").keys();
                        for key in keys {
                            let value =
                                key_and_return_temp.unwrap().get(key).unwrap();
                            expected_literal_allowed.push(key.clone());
                            expected_literal_return
                                .push(value.as_str().unwrap().to_string());
                        }
                    }

                    let input = BlockInput::new(
                        temp.get("type").unwrap().as_str().unwrap().to_string(),
                        None,
                        expected_literal_allowed,
                        expected_literal_return,
                    )
                    .unwrap();
                    inputs.push(input);
                }
                println!(">{:?}", pre_inputs);

                let inline = block_type == "inline";
                let block_type_id;
                if inline {
                    block_type_id = 1
                } else {
                    block_type_id = 0
                }

                let block = Block::new(
                    name.clone(),
                    internal_name,
                    0,
                    0,
                    block_type_id,
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    output,
                    inputs,
                    block_output_types,
                    font,
                    workspace,
                );
                if inline {
                    inline_blocks.push(block);
                } else {
                    action_blocks.push(block);
                }
            } else {
                println!("Skipping Block as {} is not supported", block_type);
                continue;
            }
        }
    }
    return (action_blocks, inline_blocks);
}
#[inline]
fn generate_random_color() -> u32 {
    return getrandom::u32().expect("Unable to generate random color");
}

// #[inline]
// fn title_case(s: &str) -> String {
//     s.split_whitespace()
//         .map(|word| {
//             let mut chars = word.chars();
//             match chars.next() {
//                 Some(first) => {
//                     first.to_uppercase().collect::<String>() + chars.as_str()
//                 }
//                 None => String::new(),
//             }
//         })
//         .collect::<Vec<_>>()
//         .join(" ")
// }
// fn internal_name_to_name(internal_name: &str) -> String {
//     let mut name = internal_name.to_string();
//     name = name.replace("_", " ");
//     //title case
//     name = title_case(&name);
//     return name;
// }

// Goal: Render 1000 Blocks at >=60 fps
// Current:
// Optimized (10_000):
// FAST_RENDER OFF: 18 fps
// FAST_RENDER ON: 25 fps

use crate::platform;

pub fn main_loop<
    F: platform::shared::Framework,
    D: platform::shared::FileSystem,
    // S: RenderSettings,
    // L: Physics,
>(
    framework: &mut F,
    file_system: &D,
    // render_settings: &S,
    // logic: &L,
) {
    let _render = crate::render::RenderSettingsPretty::new();
    let _physics = crate::logic::LogicAccurate::new();
    let mut workspaces = Vec::from([
        WorkSpace::new(&_render, &_physics),
        WorkSpace::new(&_render, &_physics),
    ]);
    let mut current_workspace_id = 0;
    let workspace_length = workspaces.len();

    let mut current_workspace = &mut workspaces[current_workspace_id];

    let icon = file_system
        .get_file_contents("src/idk.ico")
        .unwrap()
        .as_image()
        .unwrap();
    framework.set_icon(&icon.0, icon.1, icon.2);
    //println!("{:?}", icon.1);
    //platform::log("Entered main loop");
    let snap_distance = 70.0;
    let scroll_multiplier = 5.0;
    let max_fps = 1000;
    framework.set_target_fps(max_fps);
    //let target_frame_delta = mirl::time::MILLIS_PER_SEC / max_fps; // Time for one frame at the target FPS
    let mut frame_start;

    let mut delta_time;
    let mut buffer: Vec<u32>;
    let mut fps;

    let mut mouse_pos = framework.get_mouse_position().unwrap_or((0.0, 0.0));
    let mut mouse_delta;

    //let mut color_names: Vec<String> = Vec::new();
    let mut block_output_color_rgb: Vec<u32> = Vec::new();
    let mut block_output_color_names: Vec<String> = Vec::new();

    //color_names.push("bool".to_string());
    // block_output_color_rgb.push(mirl::graphics::rgb_to_u32(50, 80, 255));
    // block_output_color_names.push("bool".to_string());
    let font = platform::load_font("src/inter.ttf");

    let mut translation = HashMap::new();

    let (action_blocks, inline_blocks) = load_blocks(
        file_system,
        &mut block_output_color_names,
        &mut block_output_color_rgb,
        &mut translation,
        &font,
        current_workspace,
    );
    current_workspace.action_blocks.extend(action_blocks.iter().cloned());
    current_workspace.inline_blocks.extend(inline_blocks.iter().cloned());
    // for id in 0..1000 {
    //     blocks.push(Block::new(
    //         format!("new block {}", id + 1),
    //         getrandom::u32().expect("Unable to generate random color") as i16
    //             / 1000,
    //         getrandom::u32().expect("Unable to generate random color") as i16
    //             / 1000,
    //         0,
    //         Vec::new(),
    //         Vec::new(),
    //         Vec::new(),
    //         Vec::new(),
    //         "bool".to_string(),
    //         Vec::new(),
    //         &block_output_color_names,
    //         &font,
    //     ))
    // }

    let mut last_mouse_down;
    let mut mouse_down_temp;
    let mut mouse_down = false;

    let mut fps_list: Vec<u64> = Vec::new();

    let mut selected: Option<usize> = None;
    let mut selected_type_is_action: bool = false;

    let mut mouse_outside;
    let mut stable_fps: u64;

    let (width, height) = framework.get_size();

    buffer = mirl::render::get_empty_buffer(width, height);
    let buffer_pointer: *mut u32 = buffer.as_mut_ptr();
    let total_window_size = width * height;

    let mut last_right_down = false;
    let mut last_left_down = false;

    //platform::log("Starting main loop");
    frame_start = framework.get_time();
    while framework.is_open() {
        mirl::render::clear_screen(buffer_pointer, total_window_size);

        let mut reload_workspace = false;
        if framework.is_key_down(KeyCode::Right)
            && !last_right_down
            && workspace_length > current_workspace_id + 1
        {
            current_workspace_id += 1;
            last_right_down = true;
            reload_workspace = true
        } else {
            last_right_down = false
        }
        if framework.is_key_down(KeyCode::Left)
            && !last_left_down
            && current_workspace_id > 0
        {
            current_workspace_id -= 1;
            reload_workspace = true;
            last_left_down = true
        } else {
            last_left_down = false
        }
        if reload_workspace {
            current_workspace = &mut workspaces[current_workspace_id];
            if current_workspace.action_blocks.len() == 0 {
                current_workspace
                    .action_blocks
                    .extend(action_blocks.iter().cloned());
                current_workspace
                    .inline_blocks
                    .extend(inline_blocks.iter().cloned())
            }
        }

        // if (buffer_pointer as usize) % 16 == 0 {
        //     println!("Buffer is 16-byte aligned");
        // } else {
        //     println!("Buffer is NOT 16-byte aligned");
        // }
        // Mouse stuff and block(/camera) selection/movement
        mouse_delta = mouse_pos;
        mouse_pos = framework.get_mouse_position().unwrap_or(mouse_pos);

        mouse_delta = subtract_tuple(mouse_pos, mouse_delta);
        mouse_outside = !current_workspace.logic.is_point_in_requctangle(
            mouse_pos.0,
            mouse_pos.1,
            0.0,
            0.0,
            width as f32,
            height as f32,
        );

        mouse_down_temp = mouse_down;
        mouse_down = framework.is_mouse_down(platform::MouseButton::Left);
        last_mouse_down = mouse_down_temp && mouse_down;

        handle_mouse(
            &mut current_workspace.camera,
            framework,
            scroll_multiplier,
        );

        if !mouse_outside {
            selected = handle_or_get_selected(
                mouse_down,
                last_mouse_down,
                &mut current_workspace.action_blocks,
                &mut current_workspace.camera,
                mouse_pos,
                // mouse_delta,
                // framework,
                // scroll_multiplier,
                selected,
                current_workspace.logic,
                selected_type_is_action,
            );
            if selected.is_some() && !last_mouse_down {
                selected_type_is_action = true;
            } else {
                selected = handle_or_get_selected(
                    mouse_down,
                    last_mouse_down,
                    &mut current_workspace.inline_blocks,
                    &mut current_workspace.camera,
                    mouse_pos,
                    // mouse_delta,
                    // framework,
                    // scroll_multiplier,
                    selected,
                    current_workspace.logic,
                    selected_type_is_action,
                );
                if selected.is_some() && !last_mouse_down {
                    selected_type_is_action = false;
                }
            }
        }
        //println!("Selected: {:?}, {}", selected, selected_type_is_action);
        if selected_type_is_action {
            reorder_blocks(
                mouse_down,
                &mut current_workspace.action_blocks,
                &mut current_workspace.camera,
                mouse_delta,
                &mut selected,
            );
        } else {
            reorder_blocks(
                mouse_down,
                &mut current_workspace.inline_blocks,
                &mut current_workspace.camera,
                mouse_delta,
                &mut selected,
            );
        }
        if selected.is_some() {
            if selected.unwrap() != 0 {
                panic!("Selected is not 0 -> Reordering failed?");
            }
        }
        //############################################
        handle_and_render_action_blocks_on_screen(
            buffer_pointer,
            &width,
            &height,
            &current_workspace.camera,
            &mut current_workspace.action_blocks,
            &block_output_color_rgb,
            &font,
            current_workspace.render,
            current_workspace.logic,
        );
        handle_and_render_inline_blocks_on_screen(
            buffer_pointer,
            &width,
            &height,
            &current_workspace.camera,
            &mut current_workspace.inline_blocks,
            &block_output_color_rgb,
            &font,
            current_workspace.render,
            current_workspace.logic,
        );
        if selected_type_is_action {
            handle_connection_and_render_ghost_block(
                buffer_pointer,
                &width,
                &height,
                &current_workspace.camera,
                &current_workspace.action_blocks,
                &font,
                &selected,
                snap_distance,
                current_workspace.render,
                current_workspace.logic,
                &block_output_color_rgb,
            );
        } else {
            handle_connection_and_render_ghost_block(
                buffer_pointer,
                &width,
                &height,
                &current_workspace.camera,
                &current_workspace.inline_blocks,
                &font,
                &selected,
                snap_distance,
                current_workspace.render,
                current_workspace.logic,
                &block_output_color_rgb,
            );
        }
        let mouse_size = 4;
        let mouse_size_half = mouse_size as f32 / 2.0;

        if current_workspace.logic.is_reqtuctangle_visible_on_screen(
            mouse_pos.0 - mouse_size_half,
            mouse_pos.1 - mouse_size_half,
            mouse_size as f32,
            mouse_size as f32,
            &current_workspace.camera,
            &(width as isize),
            &(width as isize),
        ) {
            current_workspace.render.draw_circle(
                buffer_pointer,
                width,
                height,
                mouse_pos.0 as usize,
                mouse_pos.1 as usize,
                mouse_size,
                rgb_to_u32(100, 20, 200),
            );
        }
        //############################################
        framework.update(&buffer); // "Unable to update window :("
        delta_time = frame_start.get_elapsed_time();
        frame_start = framework.get_time();

        if delta_time != 0 {
            fps = mirl::time::MILLIS_PER_SEC / delta_time;
        } else {
            fps = u64::MAX;
        }

        add_item_to_max_sized_list(&mut fps_list, fps as usize / 4, fps as u64);
        if fps_list.len() == 0 {
            stable_fps = 0;
        } else {
            stable_fps = fps_list.iter().sum::<u64>() / fps_list.len() as u64;
        }

        framework.set_title(
            to_monospace_unicode(&format!(
                "Rust Window {:>4}/{:>4} FPS (Sampling {:>3}) | {:>8}x {:>8}y {:>4}z | {:>8}x {:>8}y -> {:>4} {:>4} -> {:>3} {:>3} | {} + {} = {}",
                stable_fps,
                fps,
                fps_list.len(),
                current_workspace.camera.x,
                current_workspace.camera.y,
                current_workspace.camera.z,
                current_workspace.camera.x + mouse_pos.0 as isize,
                current_workspace.camera.y + mouse_pos.1 as isize,
                mouse_pos.0,
                mouse_pos.1,
                mouse_delta.0,
                mouse_delta.1,
                current_workspace.action_blocks.len(),
                current_workspace.inline_blocks.len(),
                current_workspace.action_blocks.len()+
                current_workspace.inline_blocks.len()
            ))
            .as_str(),
        );
        // if selected.is_some() {
        //     let block = &blocks[selected.unwrap()];
        //     println!("Block Index: {}, ID: {}, Connected bottom: {}, Connected top: {}, Possible connection below: {}, Possible Connection Above: {}",
        //     selected.unwrap(), block.id,
        //     block.connected_below.get().map_or("None".to_string(), |id| id.to_string()),
        //     block.connected_top.get().map_or("None".to_string(), |id| id.to_string()),
        //     block.possible_connection_below.get().map_or("None".to_string(), |id| id.to_string()),
        //     block.possible_connection_above.get().map_or("None".to_string(), |id| id.to_string()))
        // }

        // if delta_time < target_frame_delta && false {
        //     let sleep_time = target_frame_delta - delta_time;
        //     // platform::log(&format!(
        //     //     "Delta: {}, Needed: {}, Sleeping for: {}",
        //     //     delta_time, target_frame_delta, sleep_time
        //     // ));
        //     framework.wait(sleep_time);
        // }
        //platform::log("Got through iter")
    }
}

fn to_monospace_unicode(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            // Digits 0–9
            '0'..='9' => char::from_u32(0x1D7F6 + (c as u32 - '0' as u32)),
            // Uppercase A–Z
            'A'..='Z' => char::from_u32(0x1D670 + (c as u32 - 'A' as u32)),
            // Lowercase a–z
            'a'..='z' => char::from_u32(0x1D68A + (c as u32 - 'a' as u32)),
            // Fallback: return original char
            _ => Some(c),
        })
        .map(|c| c.unwrap_or('�'))
        .collect()
}
// fn main() {
//     let width = 800;
//     let height = 600;

//     let window_name = "Rust window";
//     // Create a window with the title "Rust Window"
//     let mut window = Window::new(
//         &window_name, // Window title
//         width,        // Width
//         height,       // Height
//         WindowOptions {
//             title: true,
//             ..WindowOptions::default()
//         },
//     )
//     .expect("Unable to create window");

//     #[cfg(target_os = "windows")]
//     window.set_icon(Icon::from_str("src/idk.ico").unwrap());
// }
