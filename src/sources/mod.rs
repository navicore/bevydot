use crate::events::GraphEvent;
use std::fmt;

pub mod dot;

/// Errors that can occur during source processing
#[derive(Debug)]
pub enum SourceError {
    /// The format of the input could not be determined
    UnknownFormat,
    /// The input is not valid for the source
    InvalidInput(String),
    /// IO or connection error
    IoError(std::io::Error),
    /// Parser-specific error
    ParseError(String),
}

impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownFormat => write!(f, "Unable to determine diagram format"),
            Self::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
            Self::IoError(err) => write!(f, "IO error: {err}"),
            Self::ParseError(msg) => write!(f, "Parse error: {msg}"),
        }
    }
}

impl std::error::Error for SourceError {}

impl From<std::io::Error> for SourceError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

/// Trait for graph event sources
/// 
/// All sources (static files, live streams, etc.) implement this trait
/// to provide a unified stream of graph events.
pub trait GraphEventSource: Send + Sync {
    /// Returns a human-readable name for this source type
    fn source_name(&self) -> &'static str;
    
    /// Converts the source content into a stream of graph events
    /// 
    /// For static sources (like DOT files), this will typically return
    /// all events at once with BatchStart/BatchEnd markers.
    /// 
    /// For live sources, this may return events over time.
    fn events(&self) -> Result<Vec<GraphEvent>, SourceError>;
    
    /// Returns true if this source can handle live updates
    fn is_live(&self) -> bool {
        false
    }
}

/// Registry for managing multiple event sources
pub struct SourceRegistry {
    sources: Vec<Box<dyn GraphEventSource>>,
}

impl SourceRegistry {
    /// Creates a new empty registry
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }
    
    /// Registers a new event source
    pub fn register(&mut self, source: Box<dyn GraphEventSource>) {
        self.sources.push(source);
    }
    
    /// Returns the number of registered sources
    pub fn len(&self) -> usize {
        self.sources.len()
    }
    
    /// Returns true if no sources are registered
    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }
    
    /// Gets a source by name
    pub fn get_source(&self, name: &str) -> Option<&dyn GraphEventSource> {
        self.sources
            .iter()
            .find(|s| s.source_name().eq_ignore_ascii_case(name))
            .map(|s| s.as_ref())
    }
}

impl Default for SourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}