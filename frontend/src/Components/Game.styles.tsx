import { mergeStyleSets, mergeStyles } from '@fluentui/merge-styles';

export const mainStyle = mergeStyleSets({
  gameHeaderStyle: mergeStyles({
    margin: 0,
    backgroundColor: "#aec8eb"
  }),
  startingStyle: mergeStyles({
    margin: 0,
    backgroundColor: "#aec8eb"
  })
});
