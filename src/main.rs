use core::panic;
use glob::Pattern;
use serde::{Deserialize, Serialize, de::value::Error};
use serde_json::{self, error};
use std::fs::DirEntry;
use std::path::Path;
use std::{clone, io};
use std::{f32::consts::E, fs};

//TODO: Move these constants to within a class once that's implemented: 
//NOTE: These will be populated by the user at initial configuration upon starting a new managed directory
//==========================================//
//----- File Paths and Config Dirs ---------//
//==========================================//

//The shadow config directory where config files and cached data is stored 
const CONFIG_DIR: &str = "/home/jack/Documents/autosnt/Test_Data/Dir_Config";
const TARGET_DIR: &str = "/home/jack/Documents/autosnt/Test_Data/Dir_A";







//==========================================//
//------ File Processing Taks Structs ------//
//==========================================//

//TODO: Decide on a Final format for this:
pub struct Task {
    pub move_order: String,
    pub source_path: String,
}

//==========================================//
//------ JSON Directory Config Structs------//
//==========================================//

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    #[serde(rename = "rule_sets")]
    pub rule_sets: Vec<RuleSet>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleSet {
    pub name: String,
    pub rules: Vec<Rule>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub path: String,
    pub age: Age,
    pub include: bool,
    #[serde(rename = "access_frequency")]
    pub access_frequency: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Age {
    pub days: i64,
    pub hours: i64,
    pub minutes: i64,
    pub seconds: i64,
}

//TODO: Come up with a better name for "move_order"

/**
 * Gets all of the include/exclude rules in the currend directory
 * for the specified move order
 *
 * path: Path to the directory in question
 * move_order: string identirier of the move
 */
fn get_rules(path: String, move_order: String) -> Result<Vec<Rule>, std::io::Error> {
    //Check if Path is valid
    let rule_string: String = match fs::read_to_string(path + "autosnt.config") {
        Err(err) => return Err(err),
        Ok(result_string) => result_string,
    };
    println!("{rule_string}");
    let root: Root = match serde_json::from_str(&rule_string) {
        Err(err) => panic!("{:?}", err), //TODO: Handle this error
        Ok(result) => result,
    };

    for rule_set in root.rule_sets.iter() {
        if (rule_set.name == move_order) {
            return Ok(rule_set.rules.clone());
        }
    }

    //TODO: Handle this error with the write type and message
    return Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "File not found",
    ));
}

fn apply_rule(path: &String, rule: &Rule) -> Result<bool, bool> {
    /**TODO:
     * Check if path is a dir or a file
     * if file get age
     * check age against age portion of rule
     * apply search for rule string to path
     * return final result
     */
    let mut time_appropriate = false;
    let mut path_appropriate: bool = false;
    let mut n_accesses_approptiate: bool = false;

    let _path: &Path = Path::new(path);
    if (_path.is_dir()) {
        //Never check the time since access of a directory
        time_appropriate = true;
    } else {
        //TODO: Check the time
        time_appropriate = true;
    }

    let matcher = match Pattern::new(&rule.path) {
        Err(err) => {
            panic!("Failed to create Pattern")
        } //TODO: Handle this properly
        Ok(result) => result,
    };

    let matcher_string = rule.path.clone();

    //println!("apply_rule: path {path}");
    //println!("apply_rule: matcher {matcher_string}");

    //TODO: Investigate if this is the best way to achieve this :
    path_appropriate = matcher.matches_path(_path);
    //println!("apply_rule: path match {path_appropriate}");

    //TODO: Handle the number of accesses check properly, for now we skip it:
    n_accesses_approptiate = true;

    //If all checks return true, the rule should be applied to the supplied path
    if(time_appropriate & path_appropriate & n_accesses_approptiate){
        return Ok(rule.include.clone());
    }
    else{
        return Err(false);
    }


    
}

fn apply_rules(path: String, rules: Vec<Rule>) -> bool {
    let mut actionable: bool = true;

    //println!("apply_rules: {path}");

    for rule in rules.iter() {
        match apply_rule(&path, rule) {
            Err(_err) => {}
            Ok(result) => {
                actionable = result;
            }
        }
    }

    return actionable;
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");


    let directory: &Path = Path::new("/home/jack/Documents/autosnt/Test_Data/Dir_A");

    if directory.is_dir() {
        let result = get_directory_actions("/".to_string(), "Dir_A-Decay".to_string(), &Vec::new());
    
        match result {
            Err(_err) => {}
            Ok(rules) => {
                for rule in rules.iter() {
                    let source_path = rule.source_path.clone();
                    println!("Task path : {source_path}")
                }
            }
        }
    }

    Ok(())
}

fn get_directory_actions(
    path: String,
    move_order: String,
    current_rules: &Vec<Rule>,
) -> Result<Vec<Task>, std::io::Error> {
    //-----Return Vector of Tasks-----//
    let mut return_tasks: Vec<Task> = Vec::new();


    //println!("get_directory_actions: {path}");


    //Apply current rules to path

    if (!apply_rules(path.clone(), current_rules.clone())){
        return Ok(return_tasks);
    }


    //------Read the Directory for Files to Process -----//
    let mut target_dir_path = TARGET_DIR.to_string() + &path;
    let mut config_dir_path: String = CONFIG_DIR.to_string() + &path;

    println!("{config_dir_path}");

    let _path: &Path = Path::new(&target_dir_path);

    if _path.is_dir() {
        //TODO: Check if there are new config conditions within this directory.
        let mut dir_specific_rules: Vec<Rule> = match get_rules(config_dir_path.clone() + "/", move_order.clone()) {
            Err(_err) => Vec::new(),
            Ok(result) => result,
        };

        let mut new_rules: Vec<Rule> = Vec::new();
        new_rules.append(&mut current_rules.clone());
        new_rules.append(&mut dir_specific_rules);

        let accessor = std::fs::read_dir(_path)?;
        for entry in accessor {
            match entry {
                Err(_err) => {}
                Ok(item) => {
                    //TODO: Decide how to handle errors from recursive calls:
                    
                    let new_path: String =  match item.path().into_string() {
                        Err(_err) => panic!("Error"), //TODO: Decide how to properly handle this
                        Ok(result) => result.replace(TARGET_DIR, "")
                    };
                    return_tasks.append(&mut get_directory_actions(
                        new_path,
                        move_order.clone(),
                        &new_rules,
                    )?);
                }
            }
        }
    } else {
        // TODO: Decide what to do when the path is a file or does not exist

        //TODO: Check if it is a valid file
        return_tasks.push(Task {
            move_order: move_order,
            source_path: path,
        });
    }
    return Ok(return_tasks);
}
