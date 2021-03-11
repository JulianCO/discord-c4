#ifndef MONTECARLO_H
#define MONTECARLO_H

#include <stdint.h>

#define ILLEGAL_MOVE -1
#define EXP_CONSTANT 1.25



typedef struct {
        char n; //number of options
        unsigned char p; // options availabe. i is available iff (avail.p & 2^i)
} Options;

typedef struct {
        int t;
        int n[7];
        int w[7];   // Wins for player whose move it is right now.
                    // e.g. yellow for the node corresponding to empty board.
        int children[7];
} MCNode;

typedef struct {
        MCNode * nodes;
        int length;
        IntStack *path;
} MCTree;

// If game is unfinished, picks a move at random
// amongst all possible ones. Otherwise, returns -1.
int choose_random_move(GameState *gs);

// Assess available moves in a position
Options available_moves(GameState *gs);

int random_choice(const Options o);

// The world's dumbest evaluation function:
// play a random game and return 1 for win, 0 for
// loss. Ties are determined by coin toss.
int monte_carlo_rollout(GameState *gs);

void monte_carlo_round(MCTree *tree, GameState *gs);

// This is the droid you're looking for
int monte_carlo_best_move(GameState *gs, uint32_t);
int select_child(MCNode * node);

// Create a new node reflecting the moves available
// in the given game state
int monte_carlo_new_node(MCTree *tree, GameState *gs);

int coin_toss();

#endif
