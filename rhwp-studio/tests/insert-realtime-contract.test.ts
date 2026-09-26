import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import test from 'node:test';

const insertSource = readFileSync(resolve(import.meta.dirname, '../src/command/commands/insert.ts'), 'utf-8');
const picturePropsDialogSource = readFileSync(resolve(import.meta.dirname, '../src/ui/picture-props-dialog.ts'), 'utf-8');

test('picture object commands emit realtime resizeObject drafts for direct property writes', () => {
  assert.match(insertSource, /emitObjectPropertiesRealtimeOperation/);
  assert.match(insertSource, /kind:\s*'resizeObject'/);
  assert.match(insertSource, /objectTargets/);
  assert.match(insertSource, /emitRealtimeOperationDraftPublic/);
  assert.match(insertSource, /applyRotationDelta/);
  assert.match(insertSource, /toggleFlip/);
  assert.match(insertSource, /insert:caption-toggle/);
});

test('picture properties dialog emits realtime resizeObject drafts', () => {
  assert.match(insertSource, /picturePropsDialog\.onRealtimeOperation\s*=\s*\(draft\)\s*=>\s*ih\.emitRealtimeOperationDraftPublic\(draft\)/);
  assert.match(picturePropsDialogSource, /onRealtimeOperation/);
  assert.match(picturePropsDialogSource, /kind:\s*'resizeObject'/);
  assert.match(picturePropsDialogSource, /objectTargets/);
  assert.match(picturePropsDialogSource, /before:/);
  assert.match(picturePropsDialogSource, /after:/);
});

test('rotation collaboration follows the committed edit and excludes unsupported OLE payloads', async () => {
  const { createServer } = await import('vite');
  const vite = await createServer({
    root: resolve(import.meta.dirname, '..'), appType: 'custom', logLevel: 'silent',
    server: { middlewareMode: true },
  });
  try {
    const { insertCommands } = await vite.ssrLoadModule('/src/command/commands/insert.ts');
    const rotate = insertCommands.find((command: { id: string }) => command.id === 'insert:rotate-cw');
    assert.ok(rotate, 'rotation command exists');
    for (const [objectType, permitted, expectedOperations] of [
      ['shape', false, 0], ['shape', true, 1], ['ole', true, 0],
    ] as const) {
      let props: Record<string, unknown> = { rotationAngle: 0 };
      const operations: unknown[] = [];
      const wasm = {
        getShapeProperties: () => ({ ...props }),
        getPictureProperties: () => ({ ...props }),
        setShapeProperties: (_sec: number, _ppi: number, _ci: number, patch: Record<string, unknown>) => {
          props = { ...props, ...patch }; return { ok: true };
        },
        setPictureProperties: (_sec: number, _ppi: number, _ci: number, patch: Record<string, unknown>) => {
          props = { ...props, ...patch }; return { ok: true };
        },
      };
      const position = { sectionIndex: 0, paragraphIndex: 2, charOffset: 0 };
      const inputHandler = {
        getSelectedPictureRef: () => ({ sec: 0, ppi: 2, ci: 1, type: objectType }),
        getPositionOutsideSelectedPicture: () => position,
        getCursorPosition: () => position,
        executeOperation: (operation: any) => {
          if (!permitted) return;
          if (operation.kind === 'command') operation.command.execute(wasm);
          else operation.operation(wasm);
        },
        emitRealtimeOperationDraftPublic: (operation: unknown) => operations.push(operation),
      };
      rotate.execute({ wasm, getInputHandler: () => inputHandler });
      assert.equal(props.rotationAngle, permitted ? 90 : 0);
      assert.equal(operations.length, expectedOperations, `${objectType}, permitted=${permitted}`);
      if (expectedOperations) {
        const operation: any = operations[0];
        assert.equal(operation.kind, 'resizeObject');
        assert.equal(operation.objectTargets[0].before.rotationAngle, 0);
        assert.equal(operation.objectTargets[0].after.rotationAngle, 90);
      }
    }
  } finally {
    await vite.close();
  }
});
