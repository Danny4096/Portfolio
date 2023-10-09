use highlight_pulldown::highlight_with_theme;
use lazy_static::lazy_static;
use pulldown_cmark;
use pulldown_cmark::Options;
use regex::Regex;
use std::error::Error;
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;

const INDEX_PATH: &str = r"\srv\www\danyaal\html\index.html";
const BLOG_MD_DIR: &str = r"\srv\www\danyaal\Portfolio\blog";
const BLOG_HTML_DIR: &str = r"\srv\www\danyaal\blog\";
fn main() {
    // check if main page exists
    assert!(fs::metadata(INDEX_PATH).is_ok());

    // read main page to string
    let file_read_result = fs::read_to_string(INDEX_PATH);
    let file_contents = match file_read_result {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e); // Use eprintln to print to stderr
            return;
        }
    };

    // set dir where blog md files are stored
    println!("{:?}", get_md_paths(BLOG_MD_DIR).unwrap()[1]);

    // fetch last modified file
    let last_modified_file = std::fs::read_dir(BLOG_MD_DIR)
        .expect("Couldn't access local directory")
        .flatten() // Remove failed
        .filter(|f| f.metadata().unwrap().is_file()) // Filter out directories (only consider files)
        .max_by_key(|x| x.metadata().unwrap().modified().unwrap()); // Get the most recently modified file

    // make regex pattern object global
    lazy_static! {
        static ref PATTERN: Regex = Regex::new(r"-->(.*)<!--END").unwrap();
    }

    let fp = last_modified_file.as_ref().unwrap().to_owned();
    let mut file_name = String::from(fp.file_name().to_str().unwrap().clone());
    file_name.truncate(file_name.len() - 3);

    let file = match fs::File::open(fp.path().to_str().unwrap().clone()) {
        Ok(file) => file,
        Err(_) => panic!("Unable to read title"),
    };
    let buffer = BufReader::new(file);

    let title = title_string(buffer);

    // update the blog line in the html with the new most recent blog post title and url
    let replacement = format!(
        "-->&nbsp;<a href=\"https://blog.danyaal.xyz/{0}.html\" class=\"blogtitle hover:underline\">{1}</a><!--END",
        file_name, title
    );
    let output = PATTERN.replace(&file_contents, replacement).to_string();
    fs::write(INDEX_PATH, output).expect("could not write file!!! check perms or smth idk");

    let md_files = get_md_paths(BLOG_MD_DIR).unwrap();
    for fp in md_files {
        convmd(&fp, &BLOG_HTML_DIR);
    }
}

//get all md files in given dir
fn get_md_paths(dir: &str) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let paths = std::fs::read_dir(dir)?
        // Filter out all those directory entries which couldn't be read
        .filter_map(|res| res.ok())
        // Map the directory entries to paths
        .map(|dir_entry| dir_entry.path())
        // Filter out all paths with extensions other than `csv`
        .filter_map(|path| {
            if path.extension().map_or(false, |ext| ext == "md") {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    Ok(paths)
}

// convert a markdown file to html given a file name
fn convmd(fp: &PathBuf, out_dir: &str) -> () {
    let fp = fp;
    let markdown_input = fs::read_to_string(fp.clone()).expect("could not read md!!!");
    let filename = fp.file_stem().unwrap().to_str().unwrap();
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    let parser = pulldown_cmark::Parser::new_ext(&markdown_input, options);
    let parser = highlight_with_theme(parser, "base16-ocean.dark").unwrap();
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser.into_iter());

    let boilerplate = "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
    <link rel=\"stylesheet\" href=\"github-markdown-dark.css\">
    <style>
        .markdown-body {
            box-sizing: border-box;
            min-width: 200px;
            max-width: 980px;
            margin: 0 auto;
            padding: 45px;
        }
    
        @media (max-width: 767px) {
            .markdown-body {
                padding: 15px;
            }
        }
    
        body {
            color-scheme: dark;
            -ms-text-size-adjust: 100%;
            -webkit-text-size-adjust: 100%;
            margin: 0;
            color: #c9d1d9;
            background-color: #0d1117;
            font-family: -apple-system, BlinkMacSystemFont, \"Segoe UI\", \"Noto Sans\", Helvetica, Arial, sans-serif, \"Apple Color Emoji\", \"Segoe UI Emoji\";
            font-size: 16px;
            line-height: 1.5;
            word-wrap: break-word;
        }
    </style>
    
    <body>
    <form class=\"navbar\" action=\"https://blog.danyaal.xyz\">
            <input class=\"blog-hp\" type=\"submit\" value=\"Blog Homepage\" />
          </form>
    <article class=\"markdown-body\">";
    html_output = format!("{}{}{}", boilerplate, html_output, "</article></body>");

    fs::write(format!("{}{}.html", out_dir, filename), html_output).expect("kys");
    ()
}

// clean up title of blog post
fn title_string<R>(mut rdr: R) -> String
where
    R: BufRead,
{
    let mut first_line = String::new();

    rdr.read_line(&mut first_line).expect("Unable to read line");

    // Where do the leading hashes stop?
    let last_hash = first_line
        .char_indices()
        .skip_while(|&(_, c)| c == '#')
        .next()
        .map_or(0, |(idx, _)| idx);

    // Trim the leading hashes and any whitespace
    first_line[last_hash..].trim().into()
}
