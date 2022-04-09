#include <stdlib.h>

/* #define NULL ((void *)0) */

typedef int bool;
#define TRUE  ((bool)1)
#define FALSE ((bool)0)

#define MIN(X, Y) (((X) < (Y)) ? (X) : (Y))

int mem_cmp(
	const void *str1,
	const void *str2,
	size_t count
) {
	const unsigned char *s1 = str1;
	const unsigned char *s2 = str2;

	while (count-- > 0) {
		if (*s1++ != *s2++) {
			return s1[-1] < s2[-1] ? -1 : 1;
		}
	}

	return 0;
}

void * mem_cpy(void *dest, const void *src, size_t len) {
	char *d = dest;
	const char *s = src;

	while (len--) {
		*d++ = *s++;
	}

	return dest;
}

void * mem_set(void *s, int c, size_t n) {
	unsigned char *p = (unsigned char *)s;

	while (n--) {
		*p++ = (unsigned char) c;
	}

	return s;
}

int str_len(const char *str) {
	const char *ptr = str;

	if (!str) {
		return 0;
	}

	while (*ptr) {
		++ptr;
	}

	return ptr - str;
}

char * str_cpy(
	char * dest,
	const char * src,
	size_t len
) {
	if (!dest || !src) {
		return NULL;
	}

	if (len == 0) {
		return dest;
	}

	mem_set(dest, '\0', len + 1);
	mem_cpy(dest, src, len);

	return dest;
}

bool has_colon_or_star(const char c) {
    return (c == ':') | (c == '*');
}

bool has_star_or_slash(const char c) {
    return (c == '*') | (c == '/');
}

int position(const char * p, const char c) {
    int i;

    int len = str_len(p);

    if (p == NULL || len == 0) {
        return -1;
    }

    for (i = 0; i < len; i++) {
        if (p[i] == c) {
            return i;
        }
    }

    return -1;
}

int loc(const char * s, const char * p) {
    int i;

    int s_len = str_len(s);
    int p_len = str_len(p);

    int len = MIN(s_len, p_len);

    for (i = 0; i < len; i++) {
        if (s[i] != p[i]) {
            return i - 1;
        }
    }

    return -1;
}

struct param {
    char * key;
    char * value;
};

enum node_kind {
    node_static,
    node_parameter,
    node_catchall
};

struct node {
    enum node_kind kind;
    char * static_data;

    void * data;

    char * indices;

    struct node * nodes;
    unsigned int nodes_len;

    char ** params;
    unsigned int params_len;
};

void node_init(
    struct node * node,
    enum node_kind kind,
    char * static_data
) {
    node->kind = kind;
    node->static_data = static_data;

    node->data = NULL;

    node->indices = NULL;

    node->nodes = NULL;
    node->nodes_len = 0;

    node->params = NULL;
    node->params_len = 0;
}

void node_free(struct node * node) {
    int i;

    free(node->static_data);
    free(node->data);
    free(node->indices);

    for (i = 0; i < node->nodes_len; i++) {
        node_free(&(node->nodes[i]));
    }

    for (i = 0; i < node->params_len; i++) {
        /* node_free(&(node->params[i])); */
    }
}

void _node_swap(struct node * a, struct node * b) {
    struct node temp = *a;
    *a = *b;
    *b = temp;
}

struct node * _node_add_node(
    struct node * node,
    char c,
    enum node_kind kind,
    char * static_data
) {
    return NULL;
}

struct node *  node_add_node_static(
    struct node * node,
    char * static_data
) {
    if (static_data == NULL || str_len(static_data) == 0) {
        return node;
    }

    return _node_add_node(node, static_data[0], node_static, static_data);
}

struct node * node_add_node_dynamic(
    struct node * node,
    char c,
    enum node_kind kind,
    char * static_data
) {
    return _node_add_node(node, c, kind, static_data);
}

struct node * node_insert(
    struct node * node,
    char * p
) {
    int l;
    int p_len;
    int s_len;
    struct node * new_node;

    switch (node->kind) {
        case node_static: {
            if (str_len(node->static_data) == 0) {
                node->static_data = p;

                return node;
            } else {
                l = loc(node->static_data, p);

                p_len = str_len(p);
                s_len = str_len(node->static_data);

                if (l < s_len) {
                    node->static_data = node->static_data + l;

                    new_node = malloc(sizeof(struct node));

                    new_node->data = NULL;

                    new_node->params = NULL;
                    new_node->params_len = 0;

                    new_node->nodes = malloc(sizeof(struct node));
                    new_node->nodes_len = 1;

                    new_node->indices = malloc(sizeof(char) * 2);
                    new_node->indices[1] = node->static_data[0];
                    new_node->indices[1] = '\0';

                    new_node->kind = node_static;
                    new_node->static_data = malloc(sizeof(char) * l);
                    str_cpy(p, new_node->static_data, l-1);

                    _node_swap(node, new_node);

                    node->nodes[0] = *new_node;
                }

                if (l == p_len) {
                    return node;
                } else {
                    return node_add_node_static(node, p + l);
                }
            }
        };
        case node_parameter: {
            return node_add_node_static(node, p);
        };
        case node_catchall: {
            return node;
        };
    }
}

void node_find(
    struct node * node,
    char * p,
    struct param * params,
    unsigned int * params_len
) {}

struct path_tree {
    struct node root;
    unsigned int data_size;
    unsigned int params;
};

void path_tree_init(
    struct path_tree * tree,
    unsigned int data_size
) {
    struct node node;
    node_init(&node, node_static, "/");
    tree->root = node;
    tree->data_size = data_size;
    tree->params = 0;
}

void path_tree_insert(
    struct path_tree * tree,
    char * path,
    void * data
) {
}

bool path_tree_find(
    struct path_tree * tree,
    char * path,
    void * data,
    struct param * params,
    unsigned int * params_len
) {
    return FALSE;
}

void find_example(struct path_tree * tree) {
    void * data = NULL;
    struct param * params = NULL;
    unsigned int params_len = 0;

    path_tree_find(tree, "/", data, params, &params_len);
}

int main() {
    struct path_tree tree;

    path_tree_init(&tree, sizeof(int));

    find_example(&tree);
}
