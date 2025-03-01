/** Converts a snake_case_name to a fancy Snake Case Name */
export function killSnake(s) {
    if (!s) return '';
    return s.split('_')
        .map(word => {
            if (word.length === 0) return '';
            return word.charAt(0).toUpperCase() + word.slice(1);
        })
        .join(' ');
}
