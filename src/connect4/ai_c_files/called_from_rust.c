#include "board.h"
#include "montecarlo.h"


uint8_t __c_montecarlo_c4_ai(uint64_t red_pieces, uint64_t blue_pieces, uint32_t tree_size) {
    GameState *imported_state;
    if (game_state_new(&imported_state) != BOARD_OK) 
        return 7;
    
    uint64_t i = 1;
    for (int e = 0; e < 42; e++) {
        if (red_pieces & i)
            imported_state->board[e] = YELLOW;
        else if (blue_pieces & i)
            imported_state->board[e] = BLUE;
        else
            imported_state->board[e] = EMPTY;
        
        i = i << 1;
    }
    
    if (red_pieces & i)
        imported_state->game_status = YELLOW_TO_PLAY;
    else
        imported_state->game_status = BLUE_TO_PLAY;
        
    int ai_move = monte_carlo_best_move(imported_state, tree_size);
    
    game_state_delete(imported_state);
    
    if (0 <= ai_move && ai_move < 7)
        return ai_move;
    else
        return 7;
}

