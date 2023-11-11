import { mergeStyles } from '@fluentui/merge-styles';

export const appStyle = mergeStyles({
  textAlign: "center",
  backgroundColor: "#aec8eb",
  minHeight: "100vh",
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  justifyContent: "center",
  fontSize: "calc(10px + 2vmin)"
});
export const startingHeader = mergeStyles({
    backgroundColor: "#aec8eb",
    width: 0,
    height: 0,
})