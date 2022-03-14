import { SimpleDexEvent } from "@/eventFilter/eventTypes";

/**
 * Nodes for call-chaining for building a tree structure to filter logs.
 * `null` is not operated on and bubbled up to root of the tree.
 *
 * e.g. building a filter to filter trades that are above a certain price level
 * for a specific market between TOKEN_A and TOKEN_B:
 *
 * const matchesOnly = SDF.narrowType(isMatchOffers);
 * const a = matchesOnly.filter(e => e.trade.tokenA.equals(TOKEN_A))
 *                      .filter(e => e.trade.tokenB.equals(TOKEN_B))
 *                      .filter(e => new Decimal(e.tokenAAmount).div(new Decimal(e.tokenBAmount)).gt(LIMIT_PRICE));
 * const b = matchesOnly.filter(e => e.trade.tokenA.equals(TOKEN_B))
 *                      .filter(e => e.trade.tokenB.equals(TOKEN_A))
 *                      .filter(e => new Decimal(e.tokenBAmount).div(new Decimal(e.tokenAAmount)).gt(LIMIT_PRICE));
 * const filter = a.or(b);
 *
 * Create your own nodes that extend the `EventFilterASTNode` by overriding `executeOnData()` to define how to operate on non-null data.
 * You can then call-chain them using then(): `SDF.then(new MyAwesomeEventFilterASTNode())`
 */
export abstract class EventFilterASTNode<I, O> {
  constructor(public readonly type: string) {}

  execute(data: I | null): O | null {
    if (data === null) return null;
    return this.executeOnData(data);
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars, class-methods-use-this
  abstract executeOnData(_data: I): O | null;

  narrowType<R extends O>(
    typeGuard: (data: O) => data is R,
  ): EventFilterASTNode<I, R> {
    return this.then(new NarrowTypeNode(typeGuard));
  }

  filter(predicate: (data: O) => boolean): EventFilterASTNode<I, O> {
    return this.then(new FilterNode(predicate));
  }

  or<R>(other: EventFilterASTNode<I, R>): EventFilterASTNode<I, O | R> {
    return new OrNode(this, other);
  }

  map<R>(transform: (data: O) => R): EventFilterASTNode<I, R> {
    return this.then(new MapNode(transform));
  }

  then<R>(second: EventFilterASTNode<O, R>): EventFilterASTNode<I, R> {
    return new ThenNode(this, second);
  }

  // TODO: optimize tree
}

class IdNode<I> extends EventFilterASTNode<I, I> {
  constructor() {
    super("Id");
  }

  // eslint-disable-next-line class-methods-use-this
  executeOnData(data: I): I {
    return data;
  }
}

/**
 * SimpleDexFilter, the primitive to build event filters from
 */
export const SDF = new IdNode<SimpleDexEvent>();

class ThenNode<I1, O1, O> extends EventFilterASTNode<I1, O> {
  constructor(
    public first: EventFilterASTNode<I1, O1>,
    public second: EventFilterASTNode<O1, O>,
  ) {
    super("Then");
  }

  executeOnData(data: I1): O | null {
    return this.second.execute(this.first.execute(data));
  }
}

class FilterNode<I> extends EventFilterASTNode<I, I> {
  constructor(public predicate: (x: I) => boolean) {
    super("Filter");
  }

  executeOnData(data: I): I | null {
    return this.predicate(data) ? data : null;
  }
}

class OrNode<I, O1, O2> extends EventFilterASTNode<I, O1 | O2> {
  constructor(
    public F1: EventFilterASTNode<I, O1>,
    public F2: EventFilterASTNode<I, O2>,
  ) {
    super("Or");
  }

  executeOnData(data: I): O1 | O2 | null {
    return this.F1.execute(data) ?? this.F2.execute(data);
  }
}

class NarrowTypeNode<I, O extends I> extends EventFilterASTNode<I, O> {
  constructor(public typeGuard: (x: I) => x is O) {
    super("NarrowType");
  }

  executeOnData(data: I): O | null {
    return this.typeGuard(data) ? data : null;
  }
}

class MapNode<I, O> extends EventFilterASTNode<I, O> {
  constructor(public transform: (x: I) => O) {
    super("Map");
  }

  executeOnData(data: I): O {
    return this.transform(data);
  }
}
