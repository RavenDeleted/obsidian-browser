#include <stdio.h>
#include <sys/prctl.h>

void sandbox_enable()
{
    prctl(
        PR_SET_NO_NEW_PRIVS,
        1,
        0,
        0,
        0
    );

    printf("[sandbox] enabled\n");
}
