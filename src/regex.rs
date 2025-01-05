use std::collections::HashMap;

use regex::Regex;

struct CapturesCache {
    cache: HashMap<String, Vec<Vec<(usize, usize)>>>,
}

impl CapturesCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn get_or_init(&mut self, re: &Regex, hay: &str) -> &Vec<Vec<(usize, usize)>> {
        self.cache.entry(hay.to_owned()).or_insert_with(|| {
            re.captures_iter(hay)
                .map(|c| c.iter().flatten().map(|m| (m.start(), m.end())).collect())
                .collect()
        })
    }
}

pub struct Cache {
    cache: HashMap<String, Result<(Regex, CapturesCache), regex::Error>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn get_or_init(
        &mut self,
        re: &str,
        hay: &str,
    ) -> Result<&Vec<Vec<(usize, usize)>>, &regex::Error> {
        self.cache
            .entry(re.to_owned())
            .or_insert_with(|| Regex::new(re).map(|r| (r, CapturesCache::new())))
            .as_mut()
            .map(|(r, c)| c.get_or_init(r, hay))
            .map_err(|err| &*err)
    }
}
