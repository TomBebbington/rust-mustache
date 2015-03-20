use std::borrow::ToOwned;
use std::{fmt, str};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf, AsPath};

use compiler::Compiler;
use error::Error;
use template::{self, Template};

/// Represents the shared metadata needed to compile and render a mustache
/// template.
#[derive(Clone)]
pub struct Context {
    pub template_path: PathBuf,
    pub template_extension: String,
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Context {{ template_path: {}, template_extension: {} }}",
               self.template_path.display(),
               self.template_extension)
    }
}

impl Context {
    /// Configures a mustache context the specified path to the templates.
    pub fn new<P>(path: P) -> Context where P:AsPath {
        Context {
            template_path: path.as_path().to_owned(),
            template_extension: "mustache".to_string(),
        }
    }

    /// Compiles a template from a string
    pub fn compile<IT: Iterator<Item=char>>(&self, reader: IT) -> Template {
        let compiler = Compiler::new(self.clone(), reader);
        let (tokens, partials) = compiler.compile();

        template::new(self.clone(), tokens, partials)
    }

    /// Compiles a template from a path.
    pub fn compile_path(&self, path: &Path) -> Result<Template, Error> {
        // FIXME(#6164): This should use the file decoding tools when they are
        // written. For now we'll just read the file and treat it as UTF-8file.
        let mut path = self.template_path.join(path);
        path.set_extension(&self.template_extension);
        let mut file = try!(File::open(&path));
        let mut template = String::new();
        try!(file.read_to_string(&mut template));

        Ok(self.compile(template.chars()))
    }
}
