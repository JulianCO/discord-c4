#ifndef STACK_H
#define STACK_H

typedef struct {
        int *v;
        unsigned int length;
        unsigned int max_length;
} IntStack;

typedef enum {
        STACK_OK,
        STACK_EMPTY,
        STACK_FULL,
        STACK_ALLOCATION_FAILED
} StackError;


StackError new_int_stack(unsigned int size, IntStack **out);
StackError pop_int_stack(IntStack *stack, int *out);
StackError push_int_stack(IntStack *stack, int n);
StackError length_int_stack(IntStack *stack, unsigned int *out);
StackError peek_int_stack(IntStack *stack, int *out);
void delete_int_stack(IntStack *stack);

#endif // STACK_H

