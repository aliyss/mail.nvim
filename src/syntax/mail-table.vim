syntax clear
syntax case match

" 1. Metadata block, hidden via folding
syn region mailTableHeader start="+++" end="+++" fold

" 2. Table rows (the table is rendered with the ASCII_MARKDOWN preset)
syn match mailTableBorder /|/
syn match mailTableSeparator /-\{3,\}/
syn match mailTableFirstCell /^|\s*\zs[^|]\+\ze\s*|/

" 3. Linking
hi def link mailTableHeader    Comment
hi def link mailTableBorder    Delimiter
hi def link mailTableSeparator Delimiter
hi def link mailTableFirstCell Function

" 4. Row color coding (applied by the table renderer per cell):
"    - the header row stands out;
"    - unread email subjects are bold;
"    - flagged email subjects are yellow;
"    - answered email subjects are green;
"    - deleted email subjects are dimmed;
"    - the multi-select marker is cyan;
"    - emails with attachments show the attachment cell in magenta.
"
" `default` keeps user themes in charge when they define these groups.
hi default link MailTableHeader   Title
highlight default MailTableUnread    term=bold cterm=bold gui=bold
highlight default MailTableFlagged   ctermfg=yellow guifg=#d7af00
highlight default MailTableAnswered  ctermfg=green guifg=#00af5f
highlight default MailTableDeleted   ctermfg=darkgray guifg=#808080
highlight default MailTableSelected  ctermfg=cyan guifg=#00afaf
highlight default MailTableAttachment ctermfg=magenta guifg=#af5fff

" 4. Buffer behavior
setlocal foldmethod=syntax
setlocal foldlevel=0
setlocal foldcolumn=0
setlocal fillchars=fold:\

function! ClearMailTableFoldText()
    let l:line_count = v:foldend - v:foldstart + 1
    return "+++ Metadata (" . l:line_count . " lines) +++"
endfunction

setlocal foldtext=ClearMailTableFoldText()

hi! link Folded mailTableHeader

let b:current_syntax = "mail-table"
