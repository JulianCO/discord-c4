#include <stdlib.h>
#include <math.h>

#include "board.h"
#include "montecarlo.h"

int choose_random_move(GameState *gs) {
        Options a;
        // assert(gs != NULL);
        a = available_moves(gs);
        return random_choice(a);
}


Options available_moves(GameState *gs) {
        Options a;
        a.n = 0;
        a.p = 0;
        unsigned char j = 1;
        for (int i = 0; i < 7; i++) {
                if(game_state_is_move_legal(gs, i)) {
                        a.n += 1;
                        a.p = a.p | j;
                }
                j = j << 1;
        }
        //printf("Returning AvailableMoves with n=%d and p=%d\n", a.n, a.p);
        return a;
}

int random_choice(const Options o) {
        int x, j, i;
        if (o.n == 0)
                return -1;
        x = rand()%(o.n);
        i = 0;
        for (j = 1; j != 0; j = j << 1) {
            if (j & o.p) {
                    if (x == 0)
                            return i;
                    else
                            x--;
            }
            i++;
        }
        return -1;
}

int monte_carlo_rollout(GameState *gs) {
        int result = 0;
        int n = 0;
        // assert(gs != NULL);
        while (!game_state_game_finished(gs)) {
                game_state_play_move(gs, choose_random_move(gs));
                n++;
                result = 1 - result;
        }
        if (game_state_get_status(gs) == TIED_GAME)
                result = coin_toss();
        while (n > 0) {
                game_state_undo_move(gs);
                n--;
        }
        return result;
}

int coin_toss() {
        return rand()%2;
}

float compute_score(float w, float n, float t) {
        return (w/n) + EXP_CONSTANT*sqrt(log(t)/n);
}

int select_child(MCNode *node) {
        float highest_score = -1.0, current_score;
        Options o;
        int j = 1;
        o.n = 0; o.p = 0;

        for (int i = 0; i < 7; i++) {
                if (node->n[i] > -1) {
                        if (node->n[i] == 0)
                                current_score = 20.0;
                        else
                                current_score = compute_score((float)node->w[i],
                                                (float) node->n[i], (float) node->t);
                        if (current_score - highest_score > -0.0001 &&
                                        highest_score - current_score > -0.0001) {
                                o.n += 1;
                                o.p = o.p | j;
                        } else if (current_score > highest_score) {
                                highest_score = current_score;
                                o.n = 1;
                                o.p = j;
                        }
                }
                j = j<<1;
        }
        //printf("Selecting amongst %d choices (%d) with score: %f\n", o.n, o.p, highest_score);

        return random_choice(o);
}

int monte_carlo_best_move(GameState *gs, uint32_t tree_size) {
        // assert(gs != NULL);
        MCTree *tree = malloc(sizeof *tree);
        if (tree == NULL)
                return -1;
        tree->nodes = malloc(tree_size * sizeof(MCNode));
        if (tree->nodes == NULL)
                return -1;
        tree->length = 0;
        if (new_int_stack(42, &(tree->path)) != STACK_OK)
                return -1;

        monte_carlo_new_node(tree, gs);

        for (uint32_t i = 0; i < tree_size - 1; i++)
                monte_carlo_round(tree, gs);

        int max_n = -1;
        int best_move = -1;
        for (int i = 0; i < 7; i++) {
                if((tree->nodes[0]).n[i] > max_n) {
                        max_n = (tree->nodes[0]).n[i];
                        best_move = i;
                }
        }
        // printf("Confidence: %f\n\n", ((float)(tree->nodes[0]).w[best_move])/((float)(tree->nodes[0]).n[best_move]));
        delete_int_stack(tree->path);
        free(tree->nodes);
        free(tree);
        return best_move;
}

void monte_carlo_round(MCTree *tree, GameState *gs) {
        int current_node = 0, temp_node;
        char in_selection = 1;
        int choice;
        int simulation_result;
        int path_segment;

        /* STEP 1: Selection */
        while(in_selection) {
                choice = select_child(&(tree->nodes[current_node]));
                //printf("Selection returned %d\n", choice);
                if (choice == -1) // No children nodes
                        in_selection = 0;
                else if ((tree->nodes[current_node]).children[choice] == -1) // Node not yet created
                        in_selection = 0;
                else {
                        game_state_play_move(gs, choice);
                        push_int_stack(tree->path, current_node*7 + choice);
                        current_node = (tree->nodes[current_node]).children[choice];
                }
        }

        /* STEP 2: Expansion */
        if (choice != -1) {
                push_int_stack(tree->path, current_node*7 + choice);
                game_state_play_move(gs, choice);
                temp_node = current_node;
                current_node = monte_carlo_new_node(tree, gs);
                (tree->nodes[temp_node]).children[choice] = current_node;
        }

        /* STEP 3: Simulation */
        simulation_result = monte_carlo_rollout(gs);

        /* STEP 4: Backpropagation */
        while (pop_int_stack(tree->path, &path_segment) == STACK_OK) {
                simulation_result = 1 - simulation_result;
                choice = path_segment%7;
                current_node = path_segment/7;
                //printf("Propagating result to node %d, move %d\n", current_node, choice);
                (tree->nodes[current_node]).t += 1;
                (tree->nodes[current_node]).n[choice] += 1;
                (tree->nodes[current_node]).w[choice] += simulation_result;
                game_state_undo_move(gs);
        }
}


int monte_carlo_new_node(MCTree *tree, GameState *gs) {
        MCNode *node = (tree->nodes) + (tree->length);
        // assert(tree->length < TREE_SIZE);
        node->t = 0;
        for (int i = 0; i < 7; i++) {
                node->children[i] = -1;
                node->w[i] = 0;
                if (game_state_is_move_legal(gs, i))
                        node->n[i] = 0;
                else
                        node->n[i] = -1;
        }
        tree->length += 1;
        return (tree->length) - 1;
}


