syntax clear
syntax case match

" 1. Metadata block, hidden via folding
syn region mailDrawerHeader start="+++" end="+++" fold

" 2. Tree lines
syn match mailDrawerIcon /[▸▾]/ contained
syn match mailDrawerAccount /^\s*[▸▾]\s\S.*/ contains=mailDrawerIcon
syn match mailDrawerFolder /^\s\{2,}[▸▾]\s\S.*/ contains=mailDrawerIcon
syn match mailDrawerAction /^\s\{4,}\S.*/

" 3. Linking
hi def link mailDrawerHeader Comment
hi def link mailDrawerIcon Statement
hi def link mailDrawerAccount Title
hi def link mailDrawerFolder Function
hi def link mailDrawerAction Comment

" 4. Buffer behavior
setlocal foldmethod=syntax
setlocal foldlevel=0
setlocal foldcolumn=0
setlocal fillchars=fold:\

function! ClearMailDrawerFoldText()
    let l:line_count = v:foldend - v:foldstart + 1
    return "+++ Metadata (" . l:line_count . " lines) +++"
endfunction

setlocal foldtext=ClearMailDrawerFoldText()

hi! link Folded mailDrawerHeader

let b:current_syntax = "mail-drawer"
