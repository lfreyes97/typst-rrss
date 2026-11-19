use std::path::PathBuf;
use std::sync::Arc;

use typst::diag::FileError;
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt as _, World};
use typst_kit::datetime::Time;
use typst_kit::downloader::SystemDownloader;
use typst_kit::files::{FileStore, FsRoot, SystemFiles};
use typst_kit::fonts::FontStore;
use typst_kit::packages::SystemPackages;

use crate::Engine;

/// Entorno de una compilación. Se crea de nuevo en cada una (como un proceso
/// CLI fresco); lo caro —las fuentes— se comparte vía `Engine`.
///
/// El documento principal es siempre el texto que se le da (no se lee del
/// disco); el resto sale de `root`.
pub struct EngineWorld {
    main_id: FileId,
    /// Parseado una vez: `source()` se llama por cada consulta de span.
    pub(crate) main: Source,
    files: FileStore<SystemFiles>,
    fonts: Arc<FontStore>,
    library: LazyHash<Library>,
    time: Time,
}

impl EngineWorld {
    pub fn new(engine: &Engine, root: PathBuf, filename: &str, text: String) -> Self {
        let vpath = VirtualPath::new(filename)
            .unwrap_or_else(|_| VirtualPath::new("main.typ").expect("vpath"));
        let main_id = RootedPath::new(typst::syntax::VirtualRoot::Project, vpath).intern();
        let downloader = SystemDownloader::new(concat!("instatypst-engine/", env!("CARGO_PKG_VERSION")));
        Self {
            main_id,
            main: Source::new(main_id, text),
            files: FileStore::new(SystemFiles::new(
                FsRoot::new(root),
                SystemPackages::new(downloader),
            )),
            fonts: engine.fonts(),
            library: LazyHash::new(Library::default()),
            time: Time::system(),
        }
    }

    pub fn main_id(&self) -> FileId {
        self.main_id
    }
}

impl World for EngineWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        self.fonts.book()
    }

    fn main(&self) -> FileId {
        self.main_id
    }

    fn source(&self, id: FileId) -> Result<Source, FileError> {
        if id == self.main_id {
            Ok(self.main.clone())
        } else {
            self.files.source(id)
        }
    }

    fn file(&self, id: FileId) -> Result<Bytes, FileError> {
        if id == self.main_id {
            Ok(Bytes::from_string(self.main.text().to_string()))
        } else {
            self.files.file(id)
        }
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.font(index)
    }

    fn today(&self, offset: Option<Duration>) -> Option<Datetime> {
        self.time.today(offset)
    }
}
