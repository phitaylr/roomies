use calamine::{Reader, Xlsx, open_workbook, Data};
use std::collections::{HashMap, HashSet};
use rand::Rng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rayon::prelude::*;
use serde::{Serialize, Deserialize};
use tauri::{Emitter, Manager};
use serde_json::json;

// Add Serialize to your structs
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Person {
    pub name: String,
    pub category: String,
    pub choices: Vec<String>,
    pub avoids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Room {
    pub category: String,
    pub members: Vec<String>,
    pub max_size: usize,
}

// ... rest of your code (all the functions) ...

impl Room {
    fn new(category: String, max_size: usize) -> Room {
        Room {
            category,
            members: Vec::new(),
            max_size,
        }
    }
    
    fn add_person(&mut self, name: String) {
        self.members.push(name);
    }
    
    fn is_full(&self) -> bool {
        self.members.len() >= self.max_size
    }
    
    fn has_space(&self) -> bool {
        self.members.len() < self.max_size
    }
}

type Solution = Vec<Room>;

#[derive(Debug, Clone)]
struct RoomDistribution {
    sizes: Vec<usize>,
}

impl RoomDistribution {
    fn new(total_people: usize, max_size: usize) -> RoomDistribution {
        let mut sizes = Vec::new();
        
        if total_people == 0 {
            return RoomDistribution { sizes };
        }
        
        // Calculate number of rooms needed
        let num_rooms = (total_people + max_size - 1) / max_size;
        
        // Distribute as evenly as possible
        let base_size = total_people / num_rooms;
        let extra = total_people % num_rooms;
        
        println!("Creating distribution for {} people, max {}", total_people, max_size);
        println!("  num_rooms: {}, base_size: {}, extra: {}", num_rooms, base_size, extra);
        
        // Create rooms: some get base_size+1, others get base_size
        for i in 0..num_rooms {
            if i < extra {
                sizes.push(base_size + 1);
            } else {
                sizes.push(base_size);
            }
        }
        
        sizes.sort();
        sizes.reverse();
        
        println!("  sizes: {:?}", sizes);
        
        RoomDistribution { sizes }
    }
}




fn generate_random_solution(
    people: &[Person],
    target_distributions: &HashMap<String, RoomDistribution>,
    seed: usize,
) -> Option<Solution> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed as u64);
    let mut solution = Vec::new();
    
    // Create rooms based on target distributions
    for (category, dist) in target_distributions {
        for &size in &dist.sizes {
            solution.push(Room::new(category.clone(), size));
        }
    }
    
    let mut placed: std::collections::HashSet<String> = std::collections::HashSet::new();
    
    // Phase 1: Greedily place people with mutual choices together
let mut people_list: Vec<&Person> = people.iter().collect();

// Sort by "social connectedness" - people with more mutual friends first
people_list.sort_by_key(|p| {
    let mutual_count = p.choices.iter()
        .filter(|choice_name| {
            people.iter().any(|other| 
                &other.name == *choice_name && 
                other.category == p.category &&
                other.choices.contains(&p.name)
            )
        })
        .count();
    -(mutual_count as i32)
});

// Add some randomness to avoid always getting the same solution
if seed % 3 > 0 {
    people_list.shuffle(&mut rng);
}

for person in &people_list {
    if placed.contains(&person.name) {
        continue;
    }
    
    // Find mutual friends who aren't placed yet
    let mut mutual_friends: Vec<&Person> = Vec::new();
    for choice_name in &person.choices {
        if let Some(friend) = people.iter().find(|p| &p.name == choice_name) {
            if friend.category == person.category 
                && !placed.contains(&friend.name)
                && friend.choices.contains(&person.name) {
                mutual_friends.push(friend);
            }
        }
    }
    
    // Try to place person with at least SOME mutual friends, not necessarily all
    if !mutual_friends.is_empty() {
        // Sort mutual friends by their connectedness (place more connected people first)
        mutual_friends.sort_by_key(|f| {
            let count = person.choices.iter()
                .filter(|c| f.choices.contains(c))
                .count();
            -(count as i32)
        });
        
        // Try to fit as many mutual friends as possible
        let mut candidate_rooms: Vec<usize> = solution.iter()
            .enumerate()
            .filter(|(_, r)| r.category == person.category && r.has_space())
            .map(|(i, _)| i)
            .collect();
        
        candidate_rooms.shuffle(&mut rng);
        
        for room_idx in candidate_rooms {
            let mut group_to_place = vec![person];
            
            // Try adding mutual friends one by one
            for friend in &mutual_friends {
                if solution[room_idx].members.len() + group_to_place.len() < solution[room_idx].max_size {
                    group_to_place.push(friend);
                }
            }
            
            // Check if all in group can be added
            let mut all_valid = true;
            for member in &group_to_place {
                if !can_add_person_to_room(member, &solution[room_idx], people) {
                    all_valid = false;
                    break;
                }
            }
            
            if all_valid {
                // Place the group
                for member in &group_to_place {
                    solution[room_idx].add_person(member.name.clone());
                    placed.insert(member.name.clone());
                }
                break;
            }
        }
    }
}
    
    // Phase 2: Place remaining people, preferring rooms with their choices
    let mut remaining: Vec<&Person> = people.iter()
        .filter(|p| !placed.contains(&p.name))
        .collect();
    remaining.shuffle(&mut rng);
    
    for person in remaining {
        let mut best_rooms: Vec<(usize, usize)> = Vec::new();
        
        for (idx, room) in solution.iter().enumerate() {
            if room.category != person.category || !room.has_space() {
                continue;
            }
            
            if !can_add_person_to_room(person, room, people) {
                continue;
            }
            
            // Count how many of their choices are in this room
            let choice_count = person.choices.iter()
                .filter(|choice| room.members.contains(choice))
                .count();
            
            best_rooms.push((idx, choice_count));
        }
        
        if best_rooms.is_empty() {
            // Can't place this person
            return None;
        }
        
        // Sort by choice count (prefer rooms with more choices)
        best_rooms.sort_by_key(|(_, count)| -(*count as i32));
        
        // If there's a tie at the top, randomize among them
        let max_count = best_rooms[0].1;
        let top_rooms: Vec<usize> = best_rooms.iter()
            .filter(|(_, count)| *count == max_count)
            .map(|(idx, _)| *idx)
            .collect();
        
        let room_idx = top_rooms[rng.gen_range(0..top_rooms.len())];
        solution[room_idx].add_person(person.name.clone());
        placed.insert(person.name.clone());
    }
    
    Some(solution)
}

fn can_add_person_to_room(person: &Person, room: &Room, all_people: &[Person]) -> bool {
    if person.category != room.category {
        return false;
    }
    
    for room_member in &room.members {
        if person.avoids.contains(room_member) {
            return false;
        }
        
        if let Some(member_data) = all_people.iter().find(|p| &p.name == room_member) {
            if member_data.avoids.contains(&person.name) {
                return false;
            }
        }
    }
    
    true
}

fn count_people_without_choices(solution: &Solution, people: &[Person]) -> usize {
    let mut count = 0;
    
    for room in solution {
        for person_name in &room.members {
            if let Some(person) = people.iter().find(|p| p.name == *person_name) {
                if !person.choices.is_empty() {
                    let has_choice = person.choices.iter()
                        .any(|choice| room.members.contains(choice));
                    
                    if !has_choice {
                        count += 1;
                    }
                }
            }
        }
    }
    
    count
}

fn read_spreadsheet(filename: &str) -> Result<Vec<Person>, Box<dyn std::error::Error>> {
    let mut workbook: Xlsx<_> = open_workbook(filename)?;
    
    let range = workbook
        .worksheet_range_at(0)
        .ok_or("No worksheet found")??;
    
    let mut rows = range.rows();
    
    let headers = rows.next().ok_or("Empty spreadsheet")?;
    
    let name_col = find_column(headers, "Name")?;
    let category_col = find_column(headers, "Category")?;
    
    let choice_cols = find_columns_starting_with(headers, "Choice");
    let avoid_cols = find_columns_starting_with(headers, "Avoid");
    
    println!("Found columns: name={}, category={}, {} choices, {} avoids", 
             name_col, category_col, choice_cols.len(), avoid_cols.len());
    
    let mut people = Vec::new();
    
    for row in rows {
        let name = get_cell_as_string(row, name_col)?;
        let category = get_cell_as_string(row, category_col)?;
        
        let choices: Vec<String> = choice_cols
            .iter()
            .filter_map(|&col| get_cell_as_string(row, col).ok())
            .filter(|s| !s.is_empty())
            .collect();
        
        let avoids: Vec<String> = avoid_cols
            .iter()
            .filter_map(|&col| get_cell_as_string(row, col).ok())
            .filter(|s| !s.is_empty())
            .collect();
        
        people.push(Person {
            name,
            category,
            choices,
            avoids,
        });
    }
    
    Ok(people)
}

fn find_column(headers: &[Data], name: &str) -> Result<usize, String> {
    headers
        .iter()
        .position(|cell| {
            if let Data::String(s) = cell {
                s.trim().eq_ignore_ascii_case(name)
            } else {
                false
            }
        })
        .ok_or_else(|| format!("Column '{}' not found", name))
}

fn find_columns_starting_with(headers: &[Data], prefix: &str) -> Vec<usize> {
    headers
        .iter()
        .enumerate()
        .filter_map(|(i, cell)| {
            if let Data::String(s) = cell {
                let cell_str = s.trim().to_lowercase();
                if cell_str.starts_with(&prefix.to_lowercase()) {
                    Some(i)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

fn get_cell_as_string(row: &[Data], col: usize) -> Result<String, String> {
    row.get(col)
        .ok_or_else(|| "Column index out of bounds".to_string())
        .map(|cell| cell.to_string().trim().to_string())
}

fn score_solution(solution: &Solution, all_people: &[Person]) -> i32 {
    let mut score = 0;
    
    for room in solution {
        for person_name in &room.members {
            if let Some(person) = all_people.iter().find(|p| p.name == *person_name) {
                for choice in &person.choices {
                    if room.members.contains(choice) {
                        score += 1;
                    }
                }
            }
        }
    }
    
    score
}

fn validate_solution(solution: &Solution, all_people: &[Person]) -> Result<(), String> {
    let mut problems = Vec::new();
    
    for room in solution {
        for person_name in &room.members {
            if let Some(person) = all_people.iter().find(|p| p.name == *person_name) {
                let has_choice = person.choices.iter()
                    .any(|choice| room.members.contains(choice));
                
                if !has_choice && !person.choices.is_empty() {
                    problems.push(format!("{} has none of their {} choices in their room", 
                                         person.name, person.choices.len()));
                }
            }
        }
    }
    
    if problems.is_empty() {
        Ok(())
    } else {
        Err(format!("Solution has problems:\n  {}", problems.join("\n  ")))
    }
}

fn calculate_imbalance(solution: &Solution) -> usize {
    let mut imbalance = 0;
    
    let mut by_category: HashMap<String, Vec<usize>> = HashMap::new();
    
    for room in solution {
        by_category.entry(room.category.clone())
            .or_insert(Vec::new())
            .push(room.members.len());
    }
    
    for sizes in by_category.values() {
        if let (Some(&max), Some(&min)) = (sizes.iter().max(), sizes.iter().min()) {
            imbalance += max - min;
        }
    }
    
    imbalance
}

fn print_solution(solution: &Solution) {
    println!("\n=== Room Assignments ===");
    
    let mut by_category: HashMap<String, Vec<&Room>> = HashMap::new();
    for room in solution {
        by_category.entry(room.category.clone())
            .or_insert(Vec::new())
            .push(room);
    }
    
    for (category, rooms) in by_category {
        println!("\n{} rooms:", category);
        
        for (i, room) in rooms.iter().enumerate() {
            println!("  Room {} - {} people:", i + 1, room.members.len());
            for member in &room.members {
                println!("    - {}", member);
            }
        }
    }
}
fn analyze_constraints(people: &[Person]) {
    println!("\n=== Constraint Analysis ===");
    
    for person in people {
        if person.choices.is_empty() {
            continue;
        }
        
        // Check if any of their choices are in the same category
        let same_category_choices: Vec<&String> = person.choices.iter()
            .filter(|choice| {
                people.iter().any(|p| &p.name == *choice && p.category == person.category)
            })
            .collect();
        
        if same_category_choices.is_empty() {
            println!("WARNING: {} has no choices in their category ({})", person.name, person.category);
        }
        
        // Check if any choices avoid them or are avoided by them
        let mut blocked_choices = 0;
        for choice_name in &person.choices {
            if let Some(choice_person) = people.iter().find(|p| &p.name == choice_name) {
                if choice_person.avoids.contains(&person.name) {
                    blocked_choices += 1;
                }
            }
        }
        
        if blocked_choices > 0 {
            println!("Note: {}/{} of {}'s choices avoid them", 
                     blocked_choices, person.choices.len(), person.name);
        }
    }
}


use rayon::prelude::*;

fn random_search(
    people: &[Person],
    target_distributions: &HashMap<String, RoomDistribution>,
    num_iterations: usize,
    app_handle: &tauri::AppHandle,
) -> Option<Solution> {
    // Pre-compute mutual friend counts ONCE
    let mutual_counts: HashMap<String, usize> = people.iter()
        .map(|person| {
            let count = person.choices.iter()
                .filter(|choice_name| {
                    people.iter().any(|other| 
                        &other.name == *choice_name && 
                        other.category == person.category &&
                        other.choices.contains(&person.name)
                    )
                })
                .count();
            (person.name.clone(), count)
        })
        .collect();
    
    println!("Running {} iterations in parallel across {} threads...", 
             num_iterations, rayon::current_num_threads());
    
    // Run iterations in parallel chunks for better progress reporting
    let chunk_size = 1000;
    let num_chunks = (num_iterations + chunk_size - 1) / chunk_size;
    
    let mut best_solution: Option<Solution> = None;
    let mut best_score = i32::MIN;
    let mut best_without_choices = usize::MAX;
    
    for chunk_idx in 0..num_chunks {
        let start_iter = chunk_idx * chunk_size;
        let end_iter = ((chunk_idx + 1) * chunk_size).min(num_iterations);
        // Emit progress
      let progress = ((end_iter as f32 / num_iterations as f32) * 100.0) as u32;
let _ = app_handle.emit_to("main", "progress", progress);      

// Learn from best solution every 5 chunks
        let pair_hints = if chunk_idx > 0 && chunk_idx % 20 == 0 {
            if let Some(ref best) = best_solution {
                extract_successful_pairs(best, people)
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };
        
        // Process chunk in parallel
        let chunk_result = (start_iter..end_iter)
            .into_par_iter()
            .filter_map(|iteration| {
                generate_random_solution_fast(
                    people,
                    target_distributions,
                    &mutual_counts,
                    &pair_hints,
                    iteration
                )
            })
            .map(|solution| {
                let choice_score = score_solution(&solution, people);
                let imbalance = calculate_imbalance(&solution);
                let without_choices = count_people_without_choices(&solution, people);
                
                let score = if without_choices == 0 {
                    choice_score  - (imbalance as i32 * 1000)
                } else {
                    choice_score * 10 - (without_choices as i32 * 1000000) - (imbalance as i32 * 10)
                };
                
                (solution, score, choice_score, without_choices, imbalance)
            })
            .max_by_key(|(_, score, _, _, _)| *score);
        
        // Update best solution from this chunk
        if let Some((solution, score, choice_score, without_choices, imbalance)) = chunk_result {
            if score > best_score {
    println!("  New best: score={}, choice_score={}, imbalance={}, without_choices={}", 
             score, choice_score, imbalance, without_choices);
    
    best_score = score;
    best_without_choices = without_choices;
    best_solution = Some(solution);
    
 // Emit update
let _ = app_handle.emit_to("main", "solution_update", json!({
    "iteration": end_iter,
    "choice_score": choice_score,
    "without_choices": without_choices,
    "imbalance": imbalance,
    "total_score": score
}));
    
    if without_choices == 0 {
        println!("  Found perfect solution where everyone gets a choice!");
    }
}
        }
        
        if chunk_idx % 5 == 0 && chunk_idx > 0 {
            println!("  Completed {} iterations... (best so far: {} without choices)", 
                     end_iter, best_without_choices);
        }
    }
    
    best_solution
}

fn extract_successful_pairs(solution: &Solution, people: &[Person]) -> HashMap<(String, String), i32> {
    let mut pairs = HashMap::new();
    
    for room in solution {
        // Look at all pairs in this room
        for i in 0..room.members.len() {
            for j in (i+1)..room.members.len() {
                let person1_name = &room.members[i];
                let person2_name = &room.members[j];
                
                // Check if this is a mutual pair
                if let Some(person1) = people.iter().find(|p| &p.name == person1_name) {
                    if let Some(person2) = people.iter().find(|p| &p.name == person2_name) {
                        let is_mutual = person1.choices.contains(person2_name) && 
                                       person2.choices.contains(person1_name);
                        
                        if is_mutual {
                            // Normalize the pair order (alphabetically)
                            let pair = if person1_name < person2_name {
                                (person1_name.clone(), person2_name.clone())
                            } else {
                                (person2_name.clone(), person1_name.clone())
                            };
                            
                            *pairs.entry(pair).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
    }
    
    pairs
}

fn generate_random_solution_fast(
    people: &[Person],
    target_distributions: &HashMap<String, RoomDistribution>,
    mutual_counts: &HashMap<String, usize>,
    pair_hints: &HashMap<(String, String), i32>,
    seed: usize,
) -> Option<Solution> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed as u64);
    let mut solution = Vec::new();
    
    for (category, dist) in target_distributions {
        for &size in &dist.sizes {
            solution.push(Room::new(category.clone(), size));
        }
    }
    
    let mut placed: HashSet<String> = HashSet::new();
    
    // Phase 1: Place mutual pairs, prioritizing those in pair_hints
    let mut people_list: Vec<&Person> = people.iter().collect();
    
    if  seed % 2 == 0 {
        people_list.sort_by_key(|p| -(mutual_counts.get(&p.name).copied().unwrap_or(0) as i32));
    } else {
        people_list.shuffle(&mut rng);
    }
    
    for person in &people_list {
        if placed.contains(&person.name) {
            continue;
        }
        
        // Sort choices by hint score (if we have hints)
        let mut choices_with_scores: Vec<(&String, i32)> = person.choices.iter()
            .map(|choice_name| {
                let pair = if &person.name < choice_name {
                    (person.name.clone(), choice_name.clone())
                } else {
                    (choice_name.clone(), person.name.clone())
                };
                let score = pair_hints.get(&pair).copied().unwrap_or(0);
                (choice_name, score)
            })
            .collect();
        
        choices_with_scores.sort_by_key(|(_, score)| -score);
        
        // Try to place with mutual friends, prioritizing hinted pairs
        for (choice_name, _) in choices_with_scores {
            if placed.contains(choice_name) {
                continue;
            }
            
            if let Some(friend) = people.iter().find(|p| &p.name == choice_name) {
                if friend.category == person.category && friend.choices.contains(&person.name) {
                    let mut candidate_rooms: Vec<usize> = solution.iter()
                        .enumerate()
                        .filter(|(_, r)| r.category == person.category && 
                                         r.members.len() + 2 <= r.max_size)
                        .map(|(i, _)| i)
                        .collect();
                    
                    candidate_rooms.shuffle(&mut rng);
                    
                    for room_idx in candidate_rooms {
                        if can_add_person_to_room(person, &solution[room_idx], people) &&
                           can_add_person_to_room(friend, &solution[room_idx], people) {
                            solution[room_idx].add_person(person.name.clone());
                            solution[room_idx].add_person(friend.name.clone());
                            placed.insert(person.name.clone());
                            placed.insert(friend.name.clone());
                            break;
                        }
                    }
                    
                    if placed.contains(&person.name) {
                        break;
                    }
                }
            }
        }
    }
    
    // Phase 2: Place remaining people, preferring rooms with their choices
    let mut remaining: Vec<&Person> = people.iter()
        .filter(|p| !placed.contains(&p.name))
        .collect();
    remaining.shuffle(&mut rng);
    
    for person in remaining {
        let mut best_rooms: Vec<(usize, usize)> = Vec::new();
        
        for (idx, room) in solution.iter().enumerate() {
            if room.category != person.category || !room.has_space() {
                continue;
            }
            
            if !can_add_person_to_room(person, room, people) {
                continue;
            }
            
            let choice_count = person.choices.iter()
                .filter(|choice| room.members.contains(choice))
                .count();
            
            best_rooms.push((idx, choice_count));
        }
        
        if best_rooms.is_empty() {
            return None;
        }
        
        best_rooms.sort_by_key(|(_, count)| -(*count as i32));
        
        let max_count = best_rooms[0].1;
        let top_rooms: Vec<usize> = best_rooms.iter()
            .filter(|(_, count)| *count == max_count)
            .map(|(idx, _)| *idx)
            .collect();
        
        let room_idx = top_rooms[rng.gen_range(0..top_rooms.len())];
        solution[room_idx].add_person(person.name.clone());
        placed.insert(person.name.clone());
    }
    
    Some(solution)
}

#[derive(Serialize, Deserialize)]
pub struct SolveResult {
    pub choice_score: i32,
    pub imbalance: usize,
    pub without_choices: usize,
    pub total_rooms: usize,
    pub rooms_by_category: HashMap<String, Vec<Vec<String>>>,
pub people: Vec<Person>,
}

pub fn solve_from_bytes(
    file_bytes: Vec<u8>,
    max_room_size: usize,
    num_iterations: usize,
    app_handle: &tauri::AppHandle,
) -> Result<SolveResult, String> {
    // Write bytes to temporary file
    use std::io::Write;
    let temp_path = std::env::temp_dir().join("roomies_temp.xlsx");
    let mut file = std::fs::File::create(&temp_path)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    file.write_all(&file_bytes)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;
    drop(file);
    
    // Read spreadsheet
    let people = read_spreadsheet(temp_path.to_str().unwrap())
        .map_err(|e| format!("Failed to read spreadsheet: {}", e))?;
    
    // Count people per category
    let mut category_counts: HashMap<String, usize> = HashMap::new();
    for person in &people {
        *category_counts.entry(person.category.clone()).or_insert(0) += 1;
    }
    
    // Calculate target distributions
    let mut target_distributions: HashMap<String, RoomDistribution> = HashMap::new();
    for (category, count) in &category_counts {
        let dist = RoomDistribution::new(*count, max_room_size);
        target_distributions.insert(category.clone(), dist);
    }
    
    // Run solver
     let solution = random_search(&people, &target_distributions, num_iterations, app_handle)
        .ok_or("No valid solution found")?;
    
    // Calculate results
    let choice_score = score_solution(&solution, &people);
    let imbalance = calculate_imbalance(&solution);
    let without_choices = count_people_without_choices(&solution, &people);
    let total_rooms = solution.len();
    
    // Group rooms by category
    let mut rooms_by_category: HashMap<String, Vec<Vec<String>>> = HashMap::new();
    for room in &solution {
        rooms_by_category
            .entry(room.category.clone())
            .or_insert(Vec::new())
            .push(room.members.clone());
    }
    
    Ok(SolveResult {
        choice_score,
        imbalance,
        without_choices,
        total_rooms,
        rooms_by_category,
        people: people.clone(),
    })
}