use tracing::warn;


#[derive(Debug)]
pub struct SpanProperties {
    pub bold: bool,
    pub italics: bool,
}

#[derive(Debug)]
pub enum Span {
    Root(Vec<Span>),
    Paragraph(Vec<Span>),
    Heading(Vec<Span>),
    Emphasis(Vec<Span>),
    TaxonomicName(Vec<Span>),
    Text(String),
    BibRefCitation(String),
    NormalizedToken(String),
    PageBreakToken(String),
    PageStartToken(String),
    Authority(String),
}

impl Span {
    pub fn paragraph() -> Self {
        Self::Paragraph(Vec::new())
    }

    pub fn heading() -> Self {
        Self::Heading(Vec::new())
    }

    pub fn emphasis() -> Self {
        Self::Emphasis(Vec::new())
    }

    pub fn taxonomic_name() -> Self {
        Self::TaxonomicName(Vec::new())
    }

    pub fn text(text: &str) -> Self {
        Self::Text(text.to_string())
    }

    pub fn citation(text: &str) -> Self {
        Self::BibRefCitation(text.to_string())
    }

    pub fn normalized_token(text: &str) -> Self {
        Self::NormalizedToken(text.to_string())
    }

    pub fn page_break_token(text: &str) -> Self {
        Self::PageBreakToken(text.to_string())
    }

    pub fn page_start_token(text: &str) -> Self {
        Self::PageStartToken(text.to_string())
    }

    pub fn authority(text: &str) -> Self {
        Self::Authority(text.to_string())
    }

    pub fn push_child(&mut self, child: Span) {
        match self {
            Span::Root(children) => children.push(child),
            Span::Paragraph(children) => children.push(child),
            Span::Heading(children) => children.push(child),
            Span::Emphasis(children) => children.push(child),
            Span::TaxonomicName(children) => children.push(child),
            Span::Text(_) => warn!("Ignoring attempt to push a child into a Text span"),
            Span::BibRefCitation(_) => warn!("Ignoring attempt to push a child into a BibRefCitation span"),
            Span::NormalizedToken(_) => warn!("Ignoring attempt to push a child into a NormalizedToken span"),
            Span::PageBreakToken(_) => warn!("Ignoring attempt to push a child into a PageBreakToken span"),
            Span::PageStartToken(_) => warn!("Ignoring attempt to push a child into a PageStartToken span"),
            Span::Authority(_) => warn!("Ignoring attempt to push a child into a PageStartToken span"),
        }
    }
}


#[derive(Debug)]
pub struct SpanStack {
    stack: Vec<Span>,
}

impl SpanStack {
    pub fn new() -> SpanStack {
        let root = Span::Root(vec![]);
        SpanStack { stack: vec![root] }
    }

    pub fn push(&mut self, child: Span) {
        let commit = match child {
            Span::Text(_) => true,
            Span::BibRefCitation(_) => true,
            Span::NormalizedToken(_) => true,
            Span::PageBreakToken(_) => true,
            Span::PageStartToken(_) => true,
            Span::Authority(_) => true,
            _ => false,
        };

        self.stack.push(child);
        if commit {
            self.commit_top();
        }
    }

    pub fn pop(&mut self) -> Option<Span> {
        self.stack.pop()
    }

    /// "Closes" the span at the top of the stack and add it to the
    /// next on the stack span, which effectively becomes the parent.
    pub fn commit_top(&mut self) {
        let item = self.stack.pop().unwrap();

        match self.stack.last_mut() {
            Some(parent) => parent.push_child(item),
            None => self.stack.push(item),
        }
    }
}
