---
description: Track down and fix infinite loops in useEffect and component rendering
---

1. **Check `useEffect` Dependencies**:
   - The most common culprit is a `useEffect` that updates a state variable which is also in its dependency array.
   - **Bad Pattern:**
     ```tsx
     useEffect(() => {
       setCount(count + 1);
     }, [count]); // Depends on 'count' -> Infinite Loop!
     ```
   - **Fix:** Use the functional update form or remove the dependency if not needed.

2. **Unstable Object References**:
   - If you pass an object or array as a dependency, React compares it by reference. Creating a new object on every render causes the effect to run every time.
   - **Fix:** Wrap the object in `useMemo` or move it outside the component.
     ```tsx
     const options = useMemo(() => ({ id: 1 }), []);
     ```

3. **Use `useTraceUpdate` Hook**:
   - Copy this hook to debug exactly which prop is changing.
   ```tsx
   function useTraceUpdate(props) {
     const prev = useRef(props);
     useEffect(() => {
       const changedProps = Object.entries(props).reduce((ps, [k, v]) => {
         if (prev.current[k] !== v) ps[k] = [prev.current[k], v];
         return ps;
       }, {});
       if (Object.keys(changedProps).length > 0) {
         console.log('Changed props:', changedProps);
       }
       prev.current = props;
     });
   }
   ```

4. **Pro Tips**:
   - Install the **eslint-plugin-react-hooks** package. It will automatically warn you about missing or circular dependencies.