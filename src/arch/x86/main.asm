BITS 32

extern _multiboot
extern _code
extern _bss
extern _end
extern kernel_start
extern print_exception_stack_frame

section .multiboot
header_start:
	dd 0x1BADB002      ; MAGIC
	dd 3               ; ALIGN | MEMINFO
	dd -(3+0x1BADB002) ; CHECKSUM

	dd _multiboot
	dd _code
	dd _bss
	dd _end
	dd _start
header_end:

section .bss
align 16
stack_bottom:
	resb 16384
stack_top:

section .text
global _start
_start:
	mov esp, stack_top
	push ebx
	push eax
	cli
	call kernel_start
_halt:
	hlt
	jmp _halt

global enable_paging
enable_paging:
	push eax

	mov eax, cr0
	or eax, 0x80000000
	mov cr0, eax

	pop eax
	ret

global insl_asm
insl_asm:
	push ebp
	mov ebp, esp

	push ecx
	push edi
	push eax

	mov ecx, [ebp + 16] ; third argument, length in ecx
	mov edi, [ebp + 12] ; second argument (out) in edi
	mov dx, [ebp + 8] ; first argument (port) in dx

	rep insd

	pop eax
	pop edi
	pop ecx

	pop ebp
	ret

global outsl_asm
outsl_asm:
	push ebp
	mov ebp, esp

	push ecx
	push esi
	push eax

	mov ecx, [ebp + 16] ; third argument, length in ecx
	mov esi, [ebp + 12] ; second argument, (in) in edi
	mov dx, [ebp + 8]   ; first argument (port) in dx.

	rep outsd

	pop eax
	pop esi
	pop ecx

	pop ebp
	ret

;; fn switch_task(from: &Registers, to: &Registers)
global switch_task
switch_task:
    ; to        + 48 -> Registers { eax[+00], ebx[+04], ecx[+08], edx[+12],
    ;                               esi[+16], edi[+20], esp[+24], ebp[+28],
    ;                               eip[+32], eflags[+36], cr3[+40] }
    ;
    ; from      + 44 -> Registers { eax[+00], ebx[+04], ecx[+08], edx[+12],
    ;                               esi[+16], edi[+20], esp[+24], ebp[+28],
    ;                               eip[+32], eflags[+36], cr3[+40] }
    ;
    ; ret       + 40
    pushad         ; EAX       + 36
                   ; ECX       + 32
                   ; EDX       + 28
                   ; EBX       + 24
                   ; ESP       + 20
                   ; EBP       + 16
                   ; ESI       + 12
                   ; EDI       + 08
    pushfd         ; EFLAGS    + 04
    mov eax, cr3   ;
    push eax       ; CR3       + 00

    ;; Store current register states into `from`.
    mov eax, [esp+44]  ; [from]
    mov [eax+04], ebx  ; from.ebx
    mov [eax+08], ecx  ; from.ecx
    mov [eax+12], edx  ; from.edx
    mov [eax+16], esi  ; from.esi
    mov [eax+20], edi  ; from.edi

    mov ebx, [esp+36]  ;
    mov [eax], ebx     ; from.eax

    mov ecx, [esp+40]  ; EIP -> ECX
    mov edx, [esp+20]  ; original ESP -> EDX
    add edx, 4         ;     and remove return value from stack.
    mov esi, [esp+16]  ; EBP -> ESI
    mov edi, [esp+04]  ; EFLAGS -> EDI

    mov [eax+24], edx  ; from.esp
    mov [eax+28], esi  ; from.ebp
    mov [eax+32], ecx  ; from.eip
    mov [eax+36], edi  ; from.eflags
    pop ebx            ;
    mov [eax+40], ebx  ; from.cr3
    push ebx           ;

    ;; Load register state from `to`
    mov eax, [esp+48]  ; [to]
    mov ebx, [eax+04]  ; to.ebx
    mov ecx, [eax+08]  ; to.ecx
    mov edx, [eax+12]  ; to.edx
    mov esi, [eax+16]  ; to.esi
    mov edi, [eax+20]  ; to.edi
                       ; to.esp later
    mov ebp, [eax+28]  ; to.ebp

    ; eflags
    push eax ; save `to`
    mov eax, [eax+36]  ; to.eflags
    push eax           ;
    popf               ; load eflags
    pop eax  ; restore `to`

    ; stack
    mov esp, [eax+24]  ; to.esp

    ; cr3
    push eax ; save `to`
    mov eax, [eax+40]  ; to.cr3
    mov cr3, eax
    mov eax, [esp] ; restore and save `to`

    ; eip
    mov eax, [eax+32]  ; to.eip
    xchg eax, [esp]    ; push eip (return) and pop `to`

    mov eax, [eax]     ; to.eax

    ret

