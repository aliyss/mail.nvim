syntax clear
syntax case match

" 1. Metadata block, hidden via folding
syn region mailComposeHeader start="+++" end="+++" fold

" 2. Editable header block (To:/Cc:/Bcc:/Subject:) and quoted body
syn match mailComposeToKey /^To:\ze/
syn match mailComposeCcKey /^Cc:\ze/
syn match mailComposeBccKey /^Bcc:\ze/
syn match mailComposeSubjectKey /^Subject:\ze/
syn match mailComposeQuoted /^>.*/

" 3. Linking
hi def link mailComposeHeader    Comment
hi def link mailComposeToKey     String
hi def link mailComposeCcKey     String
hi def link mailComposeBccKey    String
hi def link mailComposeSubjectKey Title
hi def link mailComposeQuoted    Comment

" 4. Buffer behavior
setlocal foldmethod=syntax
setlocal foldlevel=0
setlocal foldcolumn=0
setlocal fillchars=fold:\

function! ClearMailComposeFoldText()
    let l:line_count = v:foldend - v:foldstart + 1
    return "+++ Metadata (" . l:line_count . " lines) +++"
endfunction

setlocal foldtext=ClearMailComposeFoldText()

hi! link Folded mailComposeHeader

let b:current_syntax = "mail-compose"
