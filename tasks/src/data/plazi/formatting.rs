use tracing::warn;


#[derive(Debug)]
pub struct PageBreakToken {
    pub id: Option<String>,
    pub page_number: String,
    pub page_id: Option<String>,
    pub start: Option<String>,
}

#[derive(Debug)]
pub struct Table;

#[derive(Debug)]
pub struct Quantity;

#[derive(Debug)]
pub struct Date;

#[derive(Debug)]
pub struct CollectingRegion;

#[derive(Debug)]
pub struct TableNote;


#[derive(Debug)]
pub struct Uri {
    pub page_number: Option<String>,
    pub page_id: Option<String>,
}

#[derive(Debug)]
pub struct Uuid {
    pub page_number: Option<String>,
    pub page_id: Option<String>,
    pub value: String,
}


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
    SmallCaps(Vec<Span>),
    TaxonomicName(Vec<Span>),
    PageBreakToken {
        attributes: PageBreakToken,
        children: Vec<Span>,
    },
    TreatmentCitationGroup(Vec<Span>),
    TreatmentCitation(Vec<Span>),
    TaxonNameAuthority(Vec<Span>),
    Uri(Vec<Span>),

    Text(String),
    Uuid(String),
    BibRefCitation(String),
    NormalizedToken(String),
    PageStartToken(String),
    Authority(String),

    Table(Vec<Span>),
    Tr(Vec<Span>),
    Th(Vec<Span>),
    Td(Vec<Span>),

    Quantity(String),
    Date(String),
    CollectingRegion(String),
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

    pub fn small_caps() -> Self {
        Self::SmallCaps(Vec::new())
    }

    pub fn taxonomic_name() -> Self {
        Self::TaxonomicName(Vec::new())
    }

    pub fn page_break_token(attributes: PageBreakToken, children: Vec<Span>) -> Self {
        Self::PageBreakToken { attributes, children }
    }

    pub fn treatment_citation_group() -> Self {
        Self::TreatmentCitationGroup(Vec::new())
    }

    pub fn treatment_citation() -> Self {
        Self::TreatmentCitation(Vec::new())
    }

    pub fn taxon_name_authority() -> Self {
        Self::TaxonNameAuthority(Vec::new())
    }

    pub fn uri(children: Vec<Span>) -> Self {
        Self::Uri(children)
    }

    pub fn text(text: &str) -> Self {
        Self::Text(text.to_string())
    }

    pub fn uuid(text: &str) -> Self {
        Self::Uuid(text.to_string())
    }

    pub fn citation(text: &str) -> Self {
        Self::BibRefCitation(text.to_string())
    }

    pub fn normalized_token(text: &str) -> Self {
        Self::NormalizedToken(text.to_string())
    }

    pub fn page_start_token(text: &str) -> Self {
        Self::PageStartToken(text.to_string())
    }

    pub fn authority(text: &str) -> Self {
        Self::Authority(text.to_string())
    }

    pub fn table() -> Self {
        Self::Table(Vec::new())
    }

    pub fn tr() -> Self {
        Self::Tr(Vec::new())
    }

    pub fn th() -> Self {
        Self::Th(Vec::new())
    }

    pub fn td() -> Self {
        Self::Td(Vec::new())
    }

    pub fn quantity(text: &str) -> Self {
        Self::Quantity(text.to_string())
    }

    pub fn date(text: &str) -> Self {
        Self::Date(text.to_string())
    }

    pub fn collecting_region(text: &str) -> Self {
        Self::CollectingRegion(text.to_string())
    }

    pub fn push_child(&mut self, child: Span) {
        use Span::*;

        match self {
            Root(children) => children.push(child),
            Paragraph(children) => children.push(child),
            Heading(children) => children.push(child),
            Emphasis(children) => children.push(child),
            SmallCaps(children) => children.push(child),
            TaxonomicName(children) => children.push(child),
            PageBreakToken { children, .. } => children.push(child),
            TreatmentCitationGroup(children) => children.push(child),
            TreatmentCitation(children) => children.push(child),
            TaxonNameAuthority(children) => children.push(child),
            Text(_) => warn!("Ignoring attempt to push a child into a Text span"),
            Uri(_) => warn!("Ignoring attempt to push a child into a Uri span"),
            Uuid(_) => warn!("Ignoring attempt to push a child into a Uuid span"),
            BibRefCitation(_) => warn!("Ignoring attempt to push a child into a BibRefCitation span"),
            NormalizedToken(_) => warn!("Ignoring attempt to push a child into a NormalizedToken span"),
            PageStartToken(_) => warn!("Ignoring attempt to push a child into a PageStartToken span"),
            Authority(_) => warn!("Ignoring attempt to push a child into a PageStartToken span"),

            Table(children) => children.push(child),
            Tr(children) => children.push(child),
            Th(children) => children.push(child),
            Td(children) => children.push(child),

            Quantity(_) => warn!("Ignoring attempt to push a child into a Quantity span"),
            CollectingRegion(_) => warn!("Ignoring attempt to push a child into a CollectingRegion span"),
            Date(_) => warn!("Ignoring attempt to push a child into a Date span"),
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
        use Span::*;

        let commit = match child {
            Text(_) => true,
            Uri(_) => true,
            Uuid(_) => true,
            BibRefCitation(_) => true,
            NormalizedToken(_) => true,
            PageBreakToken { .. } => true,
            PageStartToken(_) => true,
            Authority(_) => true,
            Quantity(_) => true,
            CollectingRegion(_) => true,
            Date(_) => true,
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

    pub fn commit_and_pop_all(&mut self) -> Vec<Span> {
        self.commit_all();

        match self.stack.pop() {
            None => vec![],
            Some(span) => match span {
                Span::Root(children) => children,
                _ => vec![],
            },
        }
    }

    /// "Closes" the span at the top of the stack and add it to the
    /// span next on the stack, which effectively becomes the parent.
    pub fn commit_top(&mut self) {
        let item = self.stack.pop().unwrap();

        match self.stack.last_mut() {
            Some(parent) => parent.push_child(item),
            None => self.stack.push(item),
        }
    }

    pub fn commit_all(&mut self) {
        // commit until only root is left
        while let Some(span) = self.stack.last_mut() {
            match span {
                Span::Root(_) => break,
                _ => self.commit_top(),
            }
        }
    }
}
