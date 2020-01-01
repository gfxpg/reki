%.co: %.s
	clang -target amdgcn--amdhsa -mcpu=gfx900 -mno-code-object-v3 -o $@ $<

all: kernel_code.co
