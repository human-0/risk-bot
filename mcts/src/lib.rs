use typed_arena::Arena;

pub mod uct;

pub trait Strategy {
    type Action: Sized + PartialEq;
    type Data: Sized;

    fn reset(&mut self);

    fn move_root(&mut self, action: &Self::Action);

    fn select(&mut self, node: &Node<Self::Action, Self::Data>) -> Selection;

    fn expand(&mut self, node: &Node<Self::Action, Self::Data>) -> Vec<(Self::Action, Self::Data)>;

    fn simulate(&mut self, node: &Node<Self::Action, Self::Data>) -> f64;

    fn backpropagate(
        &mut self,
        mut score: f64,
        mut tree: TreeWalker<'_, '_, Self::Action, Self::Data>,
    ) {
        while let Some(node) = tree.pop() {
            node.visits += 1;
            node.score += score;
            score = 1.0 - score;
        }
    }

    fn expand_root(
        &mut self,
        node: &Node<Self::Action, Self::Data>,
    ) -> Vec<(Self::Action, Self::Data)> {
        self.expand(node)
    }
}

pub enum Selection {
    Terminal(f64),
    Selection(u32),
}

#[derive(Debug)]
pub struct Node<'a, Action, Data> {
    /// The number of visits for this node
    pub visits: u32,
    /// The score for this node from the perspective of the parent
    pub score: f64,
    /// The children of the node
    pub children: &'a mut [(Action, Data, Option<Self>)],
}

impl<Action, Data> Node<'_, Action, Data> {
    fn best_score(&self) -> Option<(&Action, f64)> {
        self.children
            .iter()
            .filter_map(|(play, _, child)| child.as_ref().map(|x| (play, x.win_ratio())))
            .max_by(|(_, score1), (_, score2)| f64::total_cmp(score1, score2))
    }

    fn most_visits(&self) -> Option<(&Action, u32)> {
        self.children
            .iter()
            .filter_map(|(play, _, child)| child.as_ref().map(|x| (play, x.visits)))
            .max_by_key(|(_, visits)| *visits)
    }

    fn win_ratio(&self) -> f64 {
        self.score / f64::from(self.visits)
    }
}

// It is probably possible to get rid of this lifetime
// This probably also needs a bit of work to be sound, but it should be fine as long as we never
// hold a reference for too long.
pub struct MCTS<'a, S>
where
    S: Strategy,
{
    root: Node<'a, S::Action, S::Data>,
    strategy: S,

    #[allow(clippy::type_complexity)]
    arena: Arena<(S::Action, S::Data, Option<Node<'a, S::Action, S::Data>>)>,
}

impl<'a, S> MCTS<'a, S>
where
    S: Strategy,
{
    pub fn new(strategy: S) -> Self {
        let root = Node {
            visits: 0,
            score: 0.0,
            children: &mut [],
        };

        let mut result = Self {
            root,
            strategy,
            arena: Arena::new(),
        };

        // SAFETY: The values returned from the arena will live as long as the arena
        result.root.children = unsafe {
            let children = result
                .strategy
                .expand_root(&result.root)
                .into_iter()
                .map(|(play, data)| (play, data, None));

            &mut *(result.arena.alloc_extend(children) as *mut _)
        };

        result
    }

    pub fn move_root(&mut self, action: S::Action) {
        self.strategy.move_root(&action);

        if let Some(new_root) = self.root.children.iter_mut().find(|(x, _, _)| x == &action) {
            if let Some(root) = new_root.2.take() {
                self.root = root;
                return;
            }
        }

        let mut root = Node {
            visits: 0,
            score: 0.0,
            children: &mut [],
        };

        // SAFETY: The values returned from the arena will live as long as the arena
        root.children = unsafe {
            let children = self
                .strategy
                .expand_root(&root)
                .into_iter()
                .map(|(play, data)| (play, data, None));

            &mut *(self.arena.alloc_extend(children) as *mut _)
        };

        self.root = root;
    }

    pub fn best_score(&self) -> Option<(&S::Action, f64)> {
        self.root.best_score()
    }

    pub fn most_visits(&self) -> Option<(&S::Action, u32)> {
        self.root.most_visits()
    }

    pub fn add_node(&mut self) {
        self.strategy.reset();
        let mut tree_walker = TreeWalker::new(&mut self.root);

        // Selection
        let result = loop {
            match self.strategy.select(tree_walker.leaf()) {
                result @ Selection::Terminal(_) => break result,
                result @ Selection::Selection(index) => {
                    if tree_walker.select(index as usize).is_none() {
                        break result;
                    }
                }
            }
        };

        let score = match result {
            Selection::Terminal(score) => score,
            Selection::Selection(index) => {
                // Expansion

                // SAFETY: The values returned from the arena will live as long as the arena
                let children = unsafe {
                    let children = self
                        .strategy
                        .expand_root(tree_walker.leaf())
                        .into_iter()
                        .map(|(play, data)| (play, data, None));

                    &mut *(self.arena.alloc_extend(children) as *mut _)
                };

                tree_walker.leaf_mut().children[index as usize].2 = Some(Node {
                    visits: 0,
                    score: 0.0,
                    children,
                });

                tree_walker.select(index as usize).unwrap();

                // Simulation
                self.strategy.simulate(tree_walker.leaf())
            }
        };

        // Backpropagation
        self.strategy.backpropagate(score, tree_walker);
    }

    pub fn strategy(&self) -> &S {
        &self.strategy
    }

    pub fn root(&self) -> &Node<'a, S::Action, S::Data> {
        &self.root
    }
}

pub struct TreeWalker<'a, 'b, Action, Data = ()>
where
    'a: 'b,
{
    ptrs: Vec<*mut Node<'a, Action, Data>>,
    _phantom: std::marker::PhantomData<&'b mut Node<'a, Action, Data>>,
}

impl<'a, 'b, Action, Data> TreeWalker<'a, 'b, Action, Data>
where
    'a: 'b,
{
    fn new(root: &'b mut Node<'a, Action, Data>) -> Self {
        Self {
            ptrs: vec![root],
            _phantom: std::marker::PhantomData,
        }
    }

    fn leaf(&self) -> &Node<'a, Action, Data> {
        // SAFETY: There are no other pointers to anything reachable from root
        // since it is borrowed by Self
        unsafe { &**self.ptrs.last().unwrap() }
    }

    fn leaf_mut(&mut self) -> &mut Node<'a, Action, Data> {
        // SAFETY: There are no other pointers to anything reachable from root
        // since it is borrowed by Self
        unsafe { &mut **self.ptrs.last().unwrap() }
    }

    fn select(&mut self, index: usize) -> Option<&mut Node<'a, Action, Data>> {
        let leaf = *self.ptrs.last().unwrap();

        // SAFETY: There are no other pointers to anything reachable from root
        // since it is borrowed by Self
        let next = unsafe { (*leaf).children[index].2.as_mut()? };

        self.ptrs.push(next);
        Some(next)
    }

    pub fn pop(&mut self) -> Option<&mut Node<'a, Action, Data>> {
        // SAFETY: There are no other pointers to anything reachable from root
        // since it is borrowed by Self
        unsafe { self.ptrs.pop().map(|x| &mut *x) }
    }
}
