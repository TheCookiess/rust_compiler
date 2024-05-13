global _start
_start:
; setup stack frame
    push rbp
    mov rbp, rsp
; Program Start
    mov rax, 15
    mov rcx, 7
    mov rsi, 20
    imul rcx, rsi
    add rax, rcx
    mov rcx, 3
    mov rsi, 5
    imul rcx, rsi
    mov rsi, 8
    add rcx, rsi
    sub rax, rcx
    mov rcx, 6
    cqo
    idiv rcx
    mov rcx, 9
    mov rsi, 4
    sub rcx, rsi
    imul rax, rcx
    mov QWORD [rsp+0], rax
; Exit Program
    mov rdi, QWORD [rsp+0] ; Ident('ans')
    mov rax, 60
    syscall
