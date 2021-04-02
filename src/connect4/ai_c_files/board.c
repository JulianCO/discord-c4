#include <stdlib.h>
#include "stack.h"

#include "board.h"

BoardError game_state_new(GameState **out) {
        StackError se;
        *out = malloc(sizeof **out);
        if (*out == NULL)
                return BOARD_ALLOCATION_FAILED;
        se = new_int_stack(7*6, &((*out)->move_history));
        if (se != STACK_OK){
                free(*out);
                return BOARD_ALLOCATION_FAILED;
        }
        for (int i = 0; i < 7; i++) {
                for (int j = 0; j < 6; j++) {
                        (*out)->board[6*i + j] = EMPTY;
                }
        }
        (*out)->game_status = YELLOW_TO_PLAY;
        return BOARD_OK;
}

BoardError game_state_play_move(GameState *gs, unsigned int spot) {
        Color move_color;
        switch(gs->game_status) {
                case YELLOW_TO_PLAY:
                        move_color = YELLOW;
                        gs->game_status = BLUE_TO_PLAY;
                        break;
                case BLUE_TO_PLAY:
                        move_color = BLUE;
                        gs->game_status = YELLOW_TO_PLAY;
                        break;
                default:
                        return BOARD_GAME_OVER;
        }
        for (int j = 0; j < 6;  j++) {
                if (gs->board[6*spot+j] == EMPTY) {
                        gs->board[6*spot + j] = move_color;
                        push_int_stack(gs->move_history, 6*spot + j);
                        game_state_update_status(gs);
                        return BOARD_OK;
                }
        }
        return BOARD_COLUMN_FULL;
}

BoardError game_state_update_status(GameState *gs) {
        int x, y, i, j, k, l, n;
        if (peek_int_stack(gs->move_history, &i) != STACK_OK)
                return BOARD_GAME_EMPTY;
        j = i % 6;
        i = i/6;
        for (int d = 0; d < 4; d++) {
                if (d == 0) {
                        k = -1;
                        l = 1;
                } else if (d == 1) {
                        k = 0;
                        l = 1;
                } else if (d == 2) {
                        k = 1;
                        l = 1;
                } else if (d == 3) {
                        k = 1;
                        l = 0;
                }
                n = 0;
                x = i;
                y = j;
                while (x >= 0 && x < 7 && y >= 0 && y < 6 
                                && (gs->board[6*x + y] == gs->board[6*i+j])) {
                        n = n+1;
                        x += k;
                        y += l;
                }
                x = i;
                y = j;
                while (x >= 0 && x < 7 && y >= 0 && y < 6 
                                && (gs->board[6*x + y] == gs->board[6*i+j])) {
                        n = n+1;
                        x -= k;
                        y -= l;
                }
                if (n >= 5) {
                        switch(gs->board[6*i+j]) {
                                case YELLOW:
                                        gs->game_status = YELLOW_WON;
                                        break;
                                case BLUE:
                                        gs->game_status = BLUE_WON;
                                        break;
                                case EMPTY:
                                        //impossible
                                        break;
                        }
                        return BOARD_OK;
                }
        }
        for (x = 0; x < 7; x++) {
                if (gs->board[6*x + 5] == EMPTY)
                        return BOARD_OK;
        }
        // If we get here, then no one won and the board is full
        gs->game_status = TIED_GAME;
        return BOARD_OK;
}

GameStatus game_state_get_status(const GameState *gs) {
        return gs->game_status;
}

char game_state_is_move_legal(const GameState *gs, int i) {
        if (i < 0 || i >= 7)
                return 0;
        if (game_state_game_finished(gs)) 
                return 0;
        return gs->board[6*i + 5] == EMPTY;
}

BoardError game_state_undo_move(GameState *gs) {
        int i,j;
        if(pop_int_stack(gs->move_history, &i) != STACK_OK)
                return BOARD_GAME_EMPTY;
        j = i%6;
        i = i/6;
        switch (gs->board[6*i+j]) {
                case YELLOW:
                        gs->game_status = YELLOW_TO_PLAY;
                        break;
                case BLUE:
                        gs->game_status = BLUE_TO_PLAY;
                        break;
                case EMPTY:
                        // impossible
                        break;
        }
        gs->board[6*i+j] = EMPTY;
        return BOARD_OK;
}

char game_state_game_finished(const GameState *gs) {
        if (gs->game_status == YELLOW_TO_PLAY)
                return 0;
        if (gs->game_status == BLUE_TO_PLAY)
                return 0;
        return 1;
}

void game_state_delete(GameState *gs) {
        delete_int_stack(gs->move_history);
        free(gs);
}





