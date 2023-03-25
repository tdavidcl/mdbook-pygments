extern crate clap;
extern crate mdbook;
extern crate serde_json;

use clap::{crate_version, Arg, ArgMatches, Command};
use mdbook::{BookItem};
use mdbook::book::{Book, Chapter};
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use mdbook::errors::{Result};

use std::io;
use std::process;

use pyo3::{prelude::*, types::{IntoPyDict, PyModule}};

pub struct Config {
    
}

impl Default for Config {
    fn default() -> Config {
        Config {
            
        }
    }
}





pub fn pygments_to_hljs(input : String){


    let reference_html = r#"<pre><div class="buttons"><button class="fa fa-copy clip-button" title="Copy to clipboard" aria-label="Copy to clipboard"><i class="tooltiptext"></i></button></div><code class="language-cpp hljs"> 
    <span class="hljs-meta">#<span class="hljs-meta-keyword">include</span> <span class="hljs-meta-string">"avoidCopyMemory.hpp"</span></span>
    <span class="hljs-meta">#<span class="hljs-meta-keyword">include</span> <span class="hljs-meta-string">"aliases.hpp"</span></span>
    
    <span class="hljs-keyword">namespace</span> shamalgs::memory::details {
    
        <span class="hljs-keyword">template</span>&lt;<span class="hljs-class"><span class="hljs-keyword">class</span> <span class="hljs-title">T</span>&gt;
        <span class="hljs-title">T</span> <span class="hljs-title">AvoidCopy</span>&lt;T&gt;:</span>:extract_element(sycl::<span class="hljs-built_in">queue</span> &amp;q, sycl::buffer&lt;T&gt; &amp;buf, u32 idx) {
    
            sycl::buffer&lt;T&gt; len_value{<span class="hljs-number">1</span>};
            q.submit([&amp;](sycl::handler &amp;cgh) {
                sycl::accessor global_mem{buf, cgh, sycl::read_only};
                sycl::accessor acc_rec{len_value, cgh, sycl::write_only, sycl::no_init};
    
                u32 idx_ = idx;
    
                cgh.single_task([=]() { acc_rec[<span class="hljs-number">0</span>] = global_mem[idx_]; });
            });
    
            T ret_val;
            {
                sycl::host_accessor acc{len_value, sycl::read_only};
                ret_val = acc[<span class="hljs-number">0</span>];
            }
    
            <span class="hljs-keyword">return</span> ret_val;
        }
    
    <span class="hljs-meta">#<span class="hljs-meta-keyword">define</span> XMAC_TYPES                                                                                 \
        X(f32   )\
        X(f32_2 )\
        X(f32_3 )\
        X(f32_4 )\
        X(f32_8 )\
        X(f32_16)\
        X(f64   )\
        X(f64_2 )\
        X(f64_3 )\
        X(f64_4 )\
        X(f64_8 )\
        X(f64_16)\
        X(u32   )\
        X(u32_2 )\
        X(u32_3 )\
        X(u32_4 )\
        X(u32_8 )\
        X(u32_16)\
        X(u64   )\
        X(u64_2 )\
        X(u64_3 )\
        X(u64_4 )\
        X(u64_8 )\
        X(u64_16)</span>
    
    <span class="hljs-meta">#<span class="hljs-meta-keyword">define</span> X(_arg_) template struct AvoidCopy<span class="hljs-meta-string">&lt;_arg_&gt;;</span></span>
        XMAC_TYPES
    <span class="hljs-meta">#<span class="hljs-meta-keyword">undef</span> X</span>
    
    } <span class="hljs-comment">// namespace shamalgs::memory::details</span>
    </code></pre>"#;



}


pub fn call_pygments(tag: &str, code : &str, pymod : & PyModule) -> Option<String> {

    if let Ok(func) = pymod.getattr("highlight_block") {
        if let Ok(func_result) = func.call1((tag.to_owned(),code.to_owned(),)) {
            let result_py: &str = func_result.extract().unwrap();

            Some(result_py.to_owned())
            
        }else{
            None
        }
    }else{
        None
    }

}

pub fn colorize_block(content : &str, pymod : & PyModule) -> Option<String> {

    let mut tmp = content.split_whitespace();


    match tmp.next() {
        Some(language_tag) => {

            let (tag, code) = content.split_at(language_tag.len());

            if tag != "rust" {
                call_pygments(tag, code, pymod)
            }else{
                None
            }

        },
        None => None,
    } 

}

pub fn colorize(content: &str, pymod : & PyModule) -> Result<String> {

    let mut buffer : String = "".to_owned();

    let mut is_in_block : bool  = false;

    for part in content.split("```") {
        

        match is_in_block {
            true => {
                match colorize_block(part,pymod) {
                    Some(res) => buffer.push_str(&res),
                    None => {
                        buffer.push_str("```");
                        buffer.push_str(part);
                        buffer.push_str("```");
                    },
                }
            },
            false => buffer.push_str(part),
        }
        is_in_block = !is_in_block;
    }
    
    Ok(buffer)
    
}


/// Add a table of contents to the given chapter.
pub fn colorize_chapter(chapter: &Chapter, cfg: &Config, pymod : & PyModule) -> Result<String> {
    colorize(&chapter.content,pymod)
}

pub struct MdbookHighlighter;

impl Preprocessor for MdbookHighlighter {

    fn name(&self) -> &str {
        "pygments"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {

        let mut res = None;
        //let cfg = ctx.config.get_preprocessor(self.name()).try_into()?;
        let cfg = Config::default();

        let py_src = include_str!("pygments_handle.py");

        eprintln!("creating highlight python interop");



        pyo3::prepare_freethreaded_python();

        Python::with_gil(|py| {
            let activators = PyModule::from_code(py, py_src, 
                "pygments_handle.py", 
                "pygments_handle").unwrap();

            book.for_each_mut(|item: &mut BookItem| {
                if let Some(Err(_)) = res {
                    return;
                }

                if let BookItem::Chapter(ref mut chapter) = *item {

                    //eprintln!("{}", chapter.content);
                    res = Some(colorize_chapter(chapter, &cfg, activators).map(|md| {
                        chapter.content = md;
                    }));
                }
            });


        });

        res.unwrap_or(Ok(())).map(|_| book)
    }
}























pub fn make_app() -> Command {
    Command::new("mdbook-pygments")
        .version(crate_version!())
        .about("mdbook preprocessor to add Table of Contents")
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {

    //test_htmlgen();

    let matches = make_app().get_matches();

 
    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(sub_args);
    }else if let Err(e) = handle_preprocessing() {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn handle_preprocessing() -> Result<(), Error> {


    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    if ctx.mdbook_version != mdbook::MDBOOK_VERSION {
        eprintln!(
            "Warning: The mdbook-pygments preprocessor was built against version \
             {} of mdbook, but we're being called from version {}",
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = MdbookHighlighter.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn handle_supports(sub_args: &ArgMatches) -> ! {

    

    let renderer = sub_args
        .get_one::<String>("renderer")
        .expect("Required argument");

    let supported = MdbookHighlighter.supports_renderer(&renderer);

    // Signal whether the renderer is supported by exiting with 1 or 0.
    if supported {
        process::exit(0);
    } else {
        process::exit(1);
    }
}
