#ifndef BOARD_H
#define BOARD_H

#include "stack.h"

typedef enum {
        BOARD_OK,
        BOARD_ALLOCATION_FAILED,
        BOARD_GAME_OVER,
        BOARD_COLUMN_FULL,
        BOARD_GAME_EMPTY
} BoardError;

typedef enum {
        EMPTY,
        BLUE,
        YELLOW
} Color;

typedef enum {
        YELLOW_TO_PLAY,
        BLUE_TO_PLAY,
        YELLOW_WON,
        BLUE_WON,
        TIED_GAME
} GameStatus;


typedef struct {
        Color board[42];
        GameStatus game_status;
        IntStack *move_history;
} GameState;

BoardError game_state_new(GameState **out);
BoardError game_state_play_move(GameState *gs, unsigned int spot);
BoardError game_state_update_status(GameState *gs);
GameStatus game_state_get_status(const GameState *gs);
char game_state_is_move_legal(const GameState *gs, int i);
BoardError game_state_undo_move(GameState *gs);
char game_state_game_finished(const GameState *gs);
void game_state_delete(GameState *gs);

#endif // BOARD_H
