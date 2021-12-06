const TEST:bool = false;
#[test]
fn test() {}


fn main() {
    #[cfg(all(feature = "bass_audio", feature = "neb_audio"))] 
    panic!("\n\n!!!!!!!!!!!!!!!!!!!!!!!!!!!\nfeatures `bass_audio` and `neb_audio` cannot be used at the same time!\nTo use neb audio, disable default features\n!!!!!!!!!!!!!!!!!!!!!!!!!!!\n\n");

    // write commits file
    // read env vars from CI
    macro_rules! env {
        ($key:expr, $tst_str:expr) => {
            if TEST {
                $tst_str.to_owned()
            } else {
                match std::env::var($key) {
                    Ok(val) => val,
                    Err(_) => return,
                }
            }
        }
    }
    macro_rules! ok {
        ($thing:expr) => {
            match $thing {
                Ok(thing) => thing,
                // if theres an error while testing, panic
                Err(e) if TEST => panic!("err: {}", e),
                // otherwise exit the fn
                Err(_) => return
            }
        };
    }
    
    let id = env!("CI_PROJECT_ID", "77");
    let url = env!("CI_API_V4_URL", "https://gitlab.ayyeve.xyz/api/v4");
    let this_commit = env!("CI_COMMIT_SHA", "1bc485e2bc088d837d893cdd22a04dc92dccd95d");
    let branch = env!("CI_COMMIT_BRANCH", "multi-mode");

    let cd = std::env::current_dir().unwrap();
    let commit_file = format!(
        "{}/taiko.rs/taiko-rs-client/src/{}commits.rs", 
        env!("CI_PROJECT_DIR", if cd.ends_with("taiko-rs-client") {"../.."} else {".."}),
        if TEST {"test-"} else {""}
    );
    println!("cd:{:?}, path: {}", cd, commit_file);

    // build the query url
    let url = format!("{}/projects/{}/repository/commits?ref_name={}", url, id, branch);

    // perform the query
    let res = ok!(reqwest::blocking::get(url));
    
    // get the data
    let response_data = ok!(res.text());
    
    // convert the data
    let commits:Vec<GitCommit> = ok!(serde_json::from_str(&response_data));

    // write the commits file
    std::fs::write(
        commit_file, 
        build_commits_file(commits, this_commit)
    ).expect("error writing commits.rs");

    if TEST {panic!("")};
}


fn build_commits_file(commits: Vec<GitCommit>, commit_hash: String) -> String {
    const TUPLE_TYPE:&'static str = "&'static str";
    const TUPLE_COUNT:usize = 5;

    let mut output = String::new();
    output += &format!("pub const COMMIT_HASH: &'static str = \"{}\";\n", commit_hash);
    output += &format!("pub const COMMITS:&[({})] = &[\n", [TUPLE_TYPE; TUPLE_COUNT].join(","));

    for commit in commits.iter() {
        output += &format!(
            "    (\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"),\n",
            commit.id,
            commit.title,
            commit.message.trim(),
            commit.committed_date,
            commit.web_url
        )
    }

    output += "];";
    output
}


#[derive(serde::Deserialize)]
pub struct GitCommit {
    /// hash
    pub id: String,
    /// short hash
    pub short_id: String,
    pub title: String,
    pub author_name: String,
    pub author_email: String,
    pub authored_date: String,
    pub committer_name: String,
    pub committer_email: String,
    pub committed_date: String,
    pub created_at: String,
    pub message: String,
    pub parent_ids: Vec<String>,
    pub web_url: String
}
