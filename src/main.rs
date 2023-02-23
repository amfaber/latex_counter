use std::{path::PathBuf, ops::{RangeFrom, Range, Index}};

use clap::Parser;
use regex::RegexBuilder;

#[derive(Parser)]
struct Args {
    filename: PathBuf,
}

#[derive(Default, Debug)]
struct Section{
    start: usize,
    end: usize,
    name: String,
    chars: usize,
    words: usize,
    lines: usize,
    total_chars: usize,
    total_words: usize,
    total_lines: usize,

    parents: [Option<usize>; 2],
    level: usize,
}

impl std::fmt::Display for Section{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent = "\t".repeat(self.level);
        let words = self.words;
        let lines = self.lines;
        let chars = self.chars;
        let total_words = self.total_words;
        let total_lines = self.total_lines;
        let total_chars = self.total_chars;
        let name = &self.name;
        if (words == total_words) && (chars == total_chars) && (lines == total_lines){
            f.write_fmt(format_args!("{indent}{name}\n{indent}words: {words}, lines: {lines}, chars: {chars}\n"))
        } else {
            f.write_fmt(format_args!("{indent}{name}\n{indent}own words: {words}, lines: {lines}, chars: {chars}\n{indent}tot words: {total_words}, lines {total_lines}, chars: {total_chars}\n"))
        }
    }
}



impl From<regex::Captures<'_>> for Section{
    fn from(capture: regex::Captures) -> Self {
        let ty = capture.get(1).unwrap().as_str().to_string();
        let level = ty.match_indices("sub").count();
        Self{
            name: capture.get(2).map(|m| m.as_str()).unwrap_or_default().to_string(),
            start: capture.get(1).unwrap().start(),
            end: capture.get(0).unwrap().end(),
            level,
            ..Default::default()
        }
    }
}

fn figure_out_parents(sections: &mut Vec<Section>){
    for i in 1..sections.len(){
        let this_level = sections[i as usize].level;
        let mut j = i as usize;
        let mut parents = [None; 2];
        loop{
            j -= 1;
            match sections.get(j as usize){
                Some(Section{level, .. }) => {
                    if *level < this_level && parents[*level].is_none(){
                        parents[*level] = Some(j as usize);
                    }
                    if *level == 0{
                        break
                    }
                },
                None => break
            }
        }

        let mut_section = sections.get_mut(i as usize).unwrap();
        mut_section.parents = parents.clone();
    }
}

fn main() {
    let args = Args::parse();

    let contents = std::fs::read_to_string(&args.filename).expect(&format!(
        "Couldn't find the provided file: {}",
        args.filename
            .as_os_str()
            .to_str()
            .expect("Couldn't find the provided file, and couldn't display the fault file name")
    ));
    let re = RegexBuilder::new(r"^[^%\n]*(\\(?:sub)*section\*?)\{((?:.|\n)*?)\}").multi_line(true).build().unwrap();

    let mut sections: Vec<Section> = re.captures_iter(&contents).map(|capture| {
        capture.into()
    }).collect();

    figure_out_parents(&mut sections);

    
    let mut total = Section{
        name: "Total".to_string(),
        ..Default::default()
    };

    let mut parents: Vec<usize> = Vec::new();
    for i in 1..=sections.len(){
        let (before, after) = sections.split_at_mut(i);
        let this_section = before.last_mut().unwrap();
        let section_meat = match after.first(){
            Some(next_section) => {
                &contents[this_section.end..next_section.start]
            },
            None => &contents[this_section.end..]
        };
        let clean_lines = section_meat.lines().map(|line|{
            match line.split_once("%"){
                Some((non_comment, _comment)) => non_comment,
                None => line
            }
        }).filter(|&line| {
            (!line.starts_with(r"\")) && (line != "") && (line != "$$")
        });
        
        let chars = clean_lines.clone().map(|line| line.len()).sum();
        let lines = clean_lines.clone().count();
        let words = clean_lines.map(|line| line.split_whitespace().count()).sum();

        this_section.chars = chars;
        this_section.lines = lines;
        this_section.words = words;

        this_section.total_chars = chars;
        this_section.total_lines = lines;
        this_section.total_words = words;

        parents.clear();
        parents.extend(this_section.parents.iter().flatten());
        for parent_idx in parents.iter(){
            let parent = sections.get_mut(*parent_idx).unwrap();
            parent.total_chars += chars;
            parent.total_lines += lines;
            parent.total_words += words;
        }

        total.chars += chars;
        total.lines += lines;
        total.words += words; 
        total.total_chars += chars;
        total.total_lines += lines;
        total.total_words += words; 
        
    }

    for section in sections{
        println!("{}", &section);
    }

    println!("\n{}", total);

}
