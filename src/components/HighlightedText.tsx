/**
 * XSS-safe rendering of backend-highlighted text.
 *
 * Backend wraps search keyword matches in `<em>` tags inside `track.title`.
 * This component parses those tags into React nodes instead of using
 * `dangerouslySetInnerHTML`, so any other HTML in the input is escaped.
 *
 * Renders a fragment so the `<em>` elements become direct children of the
 * parent element, preserving `[&>em]` CSS selectors.
 */
export function HighlightedText({ text }: { text: string }) {
  const parts = text.split(/(<em>.*?<\/em>)/gs)
  return (
    <>
      {parts.map((part, i) => {
        const match = part.match(/^<em>(.*)<\/em>$/s)
        if (match) return <em key={i}>{match[1]}</em>
        return <span key={i}>{part}</span>
      })}
    </>
  )
}
