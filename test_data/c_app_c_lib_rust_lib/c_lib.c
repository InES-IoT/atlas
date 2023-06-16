static char c_lib_static_arr[] = "This is a static array in the c library.";
static char c_lib_bss_arr[64];

int c_add(int a, int b)
{
    return a + b;
}

int c_triple_mult(int a, int b, int c)
{
    return a * b * c;
}
