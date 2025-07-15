pub fn description_validator(desc: String) -> Result<(), String> {
    if desc.is_empty() {
        return Err("Description is required".into());
    } else if desc.len() < 10 {
        return Err("Description must be at least 10 characters".into());
    }

    Ok(())
}

pub fn hashtags_validator(hashtags: String) -> Result<Vec<String>, String> {
    if hashtags.is_empty() {
        return Err("Hashtags are required".into());
    }

    let hashtags: Vec<_> = hashtags
        .split(',')
        .filter_map(|s| {
            let ht = s.trim().replace('#', "");
            if ht.is_empty() {
                None
            } else {
                Some(ht)
            }
        })
        .collect();

    if hashtags.len() > 8 {
        return Err("Only a maximum of 8 hashtags are allowed".into());
    }

    Ok(hashtags)
}
