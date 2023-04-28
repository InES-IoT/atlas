// Compile with:
// ```
// rustc -O --crate-type staticlib --target thumbv7em-none-eabi --out-dir libs lib.rs
// arm-none-eabi-gcc -march=armv7e-m app.c -Llibs -llib --specs=nosys.specs -o app
// ```
// Print symbols with `arm-none-eabi-nm --print-size --size-sort --demangle app`

extern int rust_add(int a, int b);
extern int rust_triple_mult(int a, int b, int c);
extern unsigned int rust_return_array_item(int i);
extern double rust_return_mut_array_item(int i);
extern void rust_set_mut_array_item(int i, double data);

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
    unsigned int f;
    double g;
    char arr[] = "Function-local array";

    a = 2;
    b = 3;
    c = 4;

    d = add(a,b);
    d = rust_add(a,b);
    e = triple_mult(a, b, c);
    e = rust_triple_mult(a,b,c);
    f = rust_return_array_item(2);
    g = rust_return_mut_array_item(1);
    rust_set_mut_array_item(0, 1.11);
}
