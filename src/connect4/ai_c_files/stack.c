#include <stdlib.h>

#include "stack.h"

StackError new_int_stack(unsigned int size, IntStack **out) {
        *out = malloc(sizeof **out);
        if (*out == NULL)
                return STACK_ALLOCATION_FAILED;
        (*out)->v = malloc(size * sizeof(int));
        if ((*out)->v == NULL)
                return STACK_ALLOCATION_FAILED;
        (*out)->length = 0;
        (*out)->max_length = size;
        return STACK_OK;
}
StackError pop_int_stack(IntStack *stack, int *out) {
        if (stack->length == 0)
                return STACK_EMPTY;
        stack->length -= 1;
        *out = stack->v[stack->length];
        return STACK_OK;
}
StackError push_int_stack(IntStack *stack, int n) {
        if (stack->length == stack->max_length)
                return STACK_FULL;
        stack->v[stack->length] = n;
        stack->length += 1;
        return STACK_OK;
}
StackError length_int_stack(IntStack *stack, unsigned int *out) {
        *out = stack->length;
        return STACK_OK;
}
StackError peek_int_stack(IntStack *stack, int *out) {
        if (stack->length == 0)
                return STACK_EMPTY;
        *out = stack->v[stack->length -1];
        return STACK_OK;
}

void delete_int_stack(IntStack *stack) {
        free(stack->v);
        free(stack);
}


