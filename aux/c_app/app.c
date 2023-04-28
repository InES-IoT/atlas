// Compile with `arm-none-eabi-gcc -march=armv7e-m app.c --specs=nosys.specs -o app`
// Print symbols with `arm-none-eabi-nm --print-size --size-sort app`

static char c_app_static_arr[] = "This is a static array that should be placed in the initialized data section.";
static char c_app_bss_arr[128];

int add(int a, int b)
{
    return a + b;
}

static int triple_mult(int a, int b, int c)
{
    return a * b * c;
}

int main(void)
{
    int a,b,c,d,e;
    char arr[] = "Function-local array";

    a = 2;
    b = 3;
    c = 4;

    d = add(a,b);
    e = triple_mult(a, b, c);
}
