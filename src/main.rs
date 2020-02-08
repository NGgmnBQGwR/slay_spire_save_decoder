use anyhow::{Context, Result as AnyResult};

mod cache;
mod cache_enums;
mod save_editor;

fn main() -> AnyResult<()> {
    let cwd = std::env::current_dir()?;
    let cache = cache::STSCache::load_or_create_from_file_in_folder(&cwd)
        .with_context(|| format!("Failed to load STSCache from '{:?}'", cwd))?;
    println!("Cache loaded: {}", cache);

    let savefile_path = save_editor::get_save_file_path(&cwd);
    if savefile_path.is_none() {
        println!("Unable to find any save file.");
        return Ok(());
    }
    println!("Using save file {:?}", savefile_path);
    let savefile_path = savefile_path.unwrap();

    save_editor::process_file(&savefile_path, &cache)
}
