/// A customized version of `path-tree` using nightly features and vector pre-allocations.

/// The Kind of a node.
#[derive(Clone, Debug)]
pub enum NodeKind {
    /// A static node with a path
    Static(String),

    /// A named node
    Parameter,

    /// A catch-all node
    CatchAll,
}

/// A node stores kind data params indices and children nodes.
#[derive(Clone, Debug)]
pub struct Node<T> {
    kind: NodeKind,
    data: Option<T>,
    indices: Option<String>,
    nodes: Option<Vec<Self>>,
    params: Option<Vec<String>>,
}

impl<T> const Default for Node<T> {
    #[inline]
    fn default() -> Self {
        Self::new(NodeKind::Static(String::new()))
    }
}

impl<T> Node<T> {
    /// Creates a new node with a special kind.
    #[inline]
    pub const fn new(kind: NodeKind) -> Self {
        Self {
            kind,
            data: None,
            nodes: None,
            params: None,
            indices: None,
        }
    }

    fn add_node(&mut self, c: char, kind: NodeKind) -> &mut Self {
        let indices: &mut String = self.indices.get_or_insert_with(String::new);
        let nodes: &mut Vec<Node<T>> = self.nodes.get_or_insert_with(Vec::new);

        match position(indices, c) {
            Some(i) => match kind {
                NodeKind::Static(ref s) => nodes[i].insert(s),
                _ => &mut nodes[i],
            },
            None => {
                indices.push(c);
                nodes.push(Node::new(kind));

                nodes.last_mut().unwrap()
            }
        }
    }

    /// Adds a child node witch a static path.
    pub fn add_node_static(&mut self, p: &str) -> &mut Self {
        if let Some(c) = p.chars().next() {
            self.add_node(c, NodeKind::Static(p.to_owned()))
        } else {
            self
        }
    }

    /// Adds a child node witch a dynamic path.
    pub fn add_node_dynamic(&mut self, c: char, kind: NodeKind) -> &mut Self {
        self.add_node(c, kind)
    }

    /// Inserts a path into node.
    pub fn insert(&mut self, p: &str) -> &mut Self {
        match self.kind {
            NodeKind::Static(ref mut s) if s.is_empty() => {
                *s += p;
                self
            }
            NodeKind::Static(ref mut s) => {
                let l = loc_count(s, p);

                // Split node
                if l < s.len() {
                    *s = s[l..].to_owned();

                    let mut node = Node {
                        data: None,
                        params: None,
                        nodes: Some(Vec::new()),
                        indices: s.chars().next().map(|c| c.to_string()),
                        kind: NodeKind::Static(String::from(&p[0..l])),
                    };

                    ::std::mem::swap(self, &mut node);

                    (unsafe { self.nodes.as_mut().unwrap_unchecked() }).push(node);
                }

                if l == p.len() {
                    self
                } else {
                    self.add_node_static(&p[l..])
                }
            }
            NodeKind::Parameter => self.add_node_static(p),
            NodeKind::CatchAll => self,
        }
    }

    /// Returns a reference to the node corresponding to the path.
    #[inline]
    fn find<'a>(&'a self, mut p: &'a str, params: &mut Vec<&'a str>) -> Option<&'a Self> {
        match self.kind {
            NodeKind::Static(ref s) => {
                let l = loc_count(s, p);

                if l == 0 || l < s.len() {
                    None
                } else if l == s.len() && l == p.len() {
                    Some(
                        // Fixed: has only route `/*`
                        // Ended `/` `/*any`
                        if self.data.is_none() && self.indices.is_some() && s.ends_with('/') {
                            &(unsafe { self.nodes.as_ref().unwrap_unchecked() })[{
                                // this unwrap gets optimized away
                                position(self.indices.as_ref().unwrap(), '*')?
                            }]
                        } else {
                            self
                        },
                    )
                } else {
                    let indices = self.indices.as_ref()?;
                    let nodes = self.nodes.as_ref()?;

                    p = &p[l..];

                    // Static
                    if let Some(i) =
                        position(indices, unsafe { p.chars().next().unwrap_unchecked() })
                    {
                        if let Some(n) = nodes[i].find(p, params).as_mut() {
                            return Some(
                                // Ended `/` `/*any`
                                match &n.kind {
                                    NodeKind::Static(s)
                                        if n.data.is_none()
                                            && n.indices.is_some()
                                            && s.ends_with('/') =>
                                    {
                                        &(unsafe { n.nodes.as_ref().unwrap_unchecked() })[{
                                            // this unwrap gets optimized away
                                            position(n.indices.as_ref().unwrap(), '*')?
                                        }]
                                    }
                                    _ => n,
                                },
                            );
                        }
                    }

                    // Named Parameter
                    if let Some(i) = position(indices, ':') {
                        if let Some(n) = nodes[i].find(p, params).as_mut() {
                            return Some(n);
                        }
                    }

                    // Catch-All Parameter
                    if let Some(i) = position(indices, '*') {
                        if let Some(n) = nodes[i].find(p, params).as_mut() {
                            return Some(n);
                        }
                    }

                    None
                }
            }
            NodeKind::Parameter => {
                if let Some(i) = p.find('/') {
                    let indices = self.indices.as_ref()?;

                    params.push(&p[..i]);
                    p = &p[i..];

                    let n = (unsafe { self.nodes.as_ref().unwrap_unchecked() })
                        [position(indices, unsafe { p.chars().next().unwrap_unchecked() })?]
                    .find(p, params)?;

                    Some(n)
                } else {
                    params.push(p);

                    Some(self)
                }
            }
            NodeKind::CatchAll => {
                params.push(p);

                Some(self)
            }
        }
    }
}

/// A path tree.
#[derive(Clone, Debug)]
pub struct PathTree<T> {
    root: Node<T>,
    params: usize,
}

impl<T> Default for PathTree<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> PathTree<T> {
    /// Creates a new tree with a root node.
    ///
    /// The root node is a static node with `/`.
    #[inline]
    pub fn new() -> Self {
        Self {
            root: Node::new(NodeKind::Static("/".to_owned())),
            params: 0,
        }
    }

    /// Inserts a path and data into tree.
    pub fn insert(&mut self, mut path: &str, data: T) -> &mut Self {
        let mut next = true;
        let mut node = &mut self.root;
        let mut params: Option<Vec<String>> = None;

        let mut most = 0;

        path = path.trim_start_matches('/');

        if path.is_empty() {
            node.data.replace(data);
            return self;
        }

        while next {
            match path.chars().position(has_colon_or_star) {
                Some(i) => {
                    let kind: NodeKind;
                    let mut prefix = &path[..i];
                    let mut suffix = &path[i..];

                    if !prefix.is_empty() {
                        node = node.add_node_static(prefix);
                    }

                    prefix = &suffix[..1];
                    suffix = &suffix[1..];

                    let c = unsafe { prefix.chars().next().unwrap_unchecked() };

                    if c == ':' {
                        match suffix.chars().position(has_star_or_slash) {
                            Some(i) => {
                                path = &suffix[i..];
                                suffix = &suffix[..i];
                            }
                            None => {
                                next = false;
                            }
                        }

                        kind = NodeKind::Parameter;
                    } else {
                        next = false;
                        kind = NodeKind::CatchAll;
                    }

                    most += 1;

                    params.get_or_insert_with(Vec::new).push(suffix.to_owned());
                    node = node.add_node_dynamic(c, kind);
                }
                None => {
                    next = false;
                    node = node.add_node_static(path);
                }
            }
        }

        if most > self.params {
            self.params = most;
        }

        node.data = Some(data);
        node.params = params;

        self
    }

    /// Returns a reference to the node data and params corresponding to the path.
    pub fn find<'a>(&'a self, path: &'a str) -> Option<(&'a T, Vec<(&'a str, &'a str)>)> {
        let mut values = Vec::with_capacity(self.params);

        self.root.find(path, &mut values).and_then(|node| {
            node.data.as_ref().map(|data| {
                (
                    data,
                    node.params.as_ref().map_or_else(Vec::new, |params| {
                        params
                            .iter()
                            .zip(values.iter())
                            .map(|(a, b)| (a.as_str(), *b))
                            .collect()
                    }),
                )
            })
        })
    }
}

#[inline]
const fn has_colon_or_star(c: char) -> bool {
    (c == ':') | (c == '*')
}

#[inline]
const fn has_star_or_slash(c: char) -> bool {
    (c == '*') | (c == '/')
}

#[inline]
fn position(p: &str, c: char) -> Option<usize> {
    p.chars().position(|x| x == c)
}

#[inline]
fn loc_count(s: &str, p: &str) -> usize {
    s.chars()
        .zip(p.chars())
        .take_while(|(a, b)| a == b)
        .map(|(c, _)| c.len_utf8())
        .sum()
}
