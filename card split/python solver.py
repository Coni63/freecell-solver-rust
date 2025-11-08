import heapq
import random
from typing import List, Tuple, Optional, Set
from dataclasses import dataclass
from enum import IntEnum

class Suit(IntEnum):
    CLUBS = 0
    DIAMONDS = 1
    HEARTS = 2
    SPADES = 3

class Color(IntEnum):
    BLACK = 0
    RED = 1

# Mapping couleur par suite
SUIT_COLOR = {
    Suit.CLUBS: Color.BLACK,
    Suit.SPADES: Color.BLACK,
    Suit.DIAMONDS: Color.RED,
    Suit.HEARTS: Color.RED
}

@dataclass(frozen=True)
class Card:
    rank: int  # 1-13 (As à Roi)
    suit: Suit
    
    def __str__(self):
        ranks = ['', 'A', '2', '3', '4', '5', '6', '7', '8', '9', '10', 'J', 'Q', 'K']
        suits = ['♣', '♦', '♥', '♠']
        return f"{ranks[self.rank]}{suits[self.suit]}"
    
    def color(self) -> Color:
        return SUIT_COLOR[self.suit]
    
    def encode(self) -> int:
        """Encode carte en 1 byte pour hash efficace"""
        return self.rank | (self.suit << 4)
    
    @staticmethod
    def decode(val: int) -> 'Card':
        return Card(val & 0xF, Suit(val >> 4))


class FreeCellState:
    def __init__(self):
        # 8 colonnes de jeu
        self.columns: List[List[Card]] = [[] for _ in range(8)]
        
        # 4 cellules libres
        self.free_cells: List[Optional[Card]] = [None] * 4
        
        # 4 fondations (une par suite) - on stocke juste le rang max
        self.foundations: List[int] = [0, 0, 0, 0]
        
    def copy(self) -> 'FreeCellState':
        """Copie profonde de l'état"""
        new_state = FreeCellState()
        new_state.columns = [col[:] for col in self.columns]
        new_state.free_cells = self.free_cells[:]
        new_state.foundations = self.foundations[:]
        return new_state
    
    def hash_key(self) -> int:
        """Hash pour détecter les états identiques"""
        # Canonicaliser : trier les colonnes vides et cellules
        cols_data = []
        for col in self.columns:
            if col:
                cols_data.append(tuple(c.encode() for c in col))
            else:
                cols_data.append(())
        
        # Trier pour canonicaliser
        cols_data.sort()
        
        # Cellules libres triées
        free_data = tuple(sorted(c.encode() if c else 0 for c in self.free_cells))
        
        return hash((tuple(cols_data), free_data, tuple(self.foundations)))
    
    def is_won(self) -> bool:
        """Vérifie si le jeu est gagné"""
        return all(f == 13 for f in self.foundations)
    
    def count_free_cells(self) -> int:
        """Compte les cellules libres disponibles"""
        return sum(1 for c in self.free_cells if c is None)
    
    def count_empty_columns(self) -> int:
        """Compte les colonnes vides"""
        return sum(1 for col in self.columns if not col)
    
    def max_movable_sequence(self) -> int:
        """Calcule la longueur max d'une séquence déplaçable"""
        # Formule FreeCell: (1 + cellules_libres) * 2^colonnes_vides
        free = self.count_free_cells()
        empty = self.count_empty_columns()
        return (1 + free) * (2 ** empty)
    
    def can_move_to_foundation(self, card: Card) -> bool:
        """Vérifie si une carte peut aller en fondation"""
        current = self.foundations[card.suit]
        return card.rank == current + 1
    
    def can_stack_on(self, card_below: Card, card_above: Card) -> bool:
        """Vérifie si card_above peut se poser sur card_below"""
        return (card_above.rank == card_below.rank - 1 and 
                card_above.color() != card_below.color())
    
    def __str__(self):
        s = "Foundations: " + " ".join(f"{chr(ord('♣')+i)}:{self.foundations[i]}" 
                                       for i in range(4)) + "\n"
        s += "Free cells: " + " ".join(str(c) if c else "[]" for c in self.free_cells) + "\n"
        s += "\nColumns:\n"
        max_height = max(len(col) for col in self.columns) if any(self.columns) else 0
        for row in range(max_height):
            for col in self.columns:
                if row < len(col):
                    s += f"{str(col[row]):4}"
                else:
                    s += "    "
            s += "\n"
        return s


class FreeCellSolver:
    def __init__(self, initial_state: FreeCellState):
        self.initial = initial_state
        self.visited: Set[int] = set()
        self.nodes_explored = 0
        
    def heuristic(self, state: FreeCellState) -> float:
        """Heuristique pour A*: estimation du coût restant"""
        score = 0.0
        
        # Cartes pas encore en fondation (poids principal)
        cards_remaining = 0
        for col in state.columns:
            cards_remaining += len(col)
        for cell in state.free_cells:
            if cell:
                cards_remaining += 1
        
        score += cards_remaining * 1.0
        
        # Bonus pour séquences bien ordonnées dans colonnes
        for col in state.columns:
            for i in range(len(col) - 1):
                if state.can_stack_on(col[i], col[i+1]):
                    score -= 0.3
        
        # Pénalité pour cellules libres occupées
        score += (4 - state.count_free_cells()) * 0.5
        
        # Pénalité pour cartes bloquées (sous des cartes plus hautes)
        for col in state.columns:
            for i in range(len(col) - 1):
                if col[i].rank < col[i+1].rank:
                    score += 0.5
        
        return score
    
    def get_moves(self, state: FreeCellState) -> List[Tuple[str, ...]]:
        """Génère tous les mouvements légaux depuis cet état"""
        moves = []
        
        # 1. Mouvements automatiques vers fondations (toujours bons)
        for i, col in enumerate(state.columns):
            if col and state.can_move_to_foundation(col[-1]):
                moves.append(('col_to_found', i))
        
        for i, card in enumerate(state.free_cells):
            if card and state.can_move_to_foundation(card):
                moves.append(('free_to_found', i))
        
        # 2. Colonne vers colonne
        for src in range(8):
            if not state.columns[src]:
                continue
            
            # Calculer la longueur de la séquence déplaçable
            seq_len = 1
            for i in range(len(state.columns[src]) - 1, 0, -1):
                if state.can_stack_on(state.columns[src][i-1], state.columns[src][i]):
                    seq_len += 1
                else:
                    break
            
            seq_len = min(seq_len, state.max_movable_sequence())
            
            for dst in range(8):
                if src == dst:
                    continue
                
                # Vers colonne vide
                if not state.columns[dst]:
                    # Ne déplacer vers vide que si ça libère quelque chose
                    if len(state.columns[src]) > 1:
                        moves.append(('col_to_col', src, dst, seq_len))
                # Vers carte compatible
                elif state.can_stack_on(state.columns[dst][-1], state.columns[src][-1]):
                    moves.append(('col_to_col', src, dst, 1))
        
        # 3. Colonne vers cellule libre
        for i, col in enumerate(state.columns):
            if col:
                for j, cell in enumerate(state.free_cells):
                    if cell is None:
                        moves.append(('col_to_free', i, j))
                        break  # Une seule cellule suffit
        
        # 4. Cellule libre vers colonne
        for i, card in enumerate(state.free_cells):
            if card:
                for j, col in enumerate(state.columns):
                    if not col or state.can_stack_on(col[-1], card):
                        moves.append(('free_to_col', i, j))
        
        return moves
    
    def apply_move(self, state: FreeCellState, move: Tuple[str, ...]) -> FreeCellState:
        """Applique un mouvement et retourne le nouvel état"""
        new_state = state.copy()
        move_type = move[0]
        
        if move_type == 'col_to_found':
            src = move[1]
            card = new_state.columns[src].pop()
            new_state.foundations[card.suit] = card.rank
            
        elif move_type == 'free_to_found':
            src = move[1]
            card = new_state.free_cells[src]
            new_state.free_cells[src] = None
            new_state.foundations[card.suit] = card.rank
            
        elif move_type == 'col_to_col':
            src, dst, count = move[1], move[2], move[3]
            cards = new_state.columns[src][-count:]
            new_state.columns[src] = new_state.columns[src][:-count]
            new_state.columns[dst].extend(cards)
            
        elif move_type == 'col_to_free':
            src, dst = move[1], move[2]
            card = new_state.columns[src].pop()
            new_state.free_cells[dst] = card
            
        elif move_type == 'free_to_col':
            src, dst = move[1], move[2]
            card = new_state.free_cells[src]
            new_state.free_cells[src] = None
            new_state.columns[dst].append(card)
        
        return new_state
    
    def solve(self, max_nodes: int = 100000) -> Optional[List[Tuple[str, ...]]]:
        """Résout le FreeCell avec A*"""
        # Priority queue: (f_score, counter, state, path)
        # Le counter évite la comparaison entre états
        start_h = self.heuristic(self.initial)
        counter = 0
        heap = [(start_h, counter, self.initial, [])]
        
        self.visited.add(self.initial.hash_key())
        self.nodes_explored = 0
        
        while heap and self.nodes_explored < max_nodes:
            f_score, _, state, path = heapq.heappop(heap)
            g_score = len(path)
            self.nodes_explored += 1
            
            if self.nodes_explored % 1000 == 0:
                print(f"Explored: {self.nodes_explored}, Queue: {len(heap)}, "
                      f"Path: {len(path)}, H: {f_score - g_score:.1f}")
            
            if state.is_won():
                print(f"\n✓ Solution trouvée en {len(path)} coups!")
                print(f"Nœuds explorés: {self.nodes_explored}")
                return path
            
            # Générer les mouvements
            for move in self.get_moves(state):
                new_state = self.apply_move(state, move)
                state_hash = new_state.hash_key()
                
                if state_hash not in self.visited:
                    self.visited.add(state_hash)
                    new_g = g_score + 1
                    new_h = self.heuristic(new_state)
                    new_f = new_g + new_h
                    
                    counter += 1
                    heapq.heappush(heap, (new_f, counter, new_state, path + [move]))
        
        print(f"\n✗ Pas de solution trouvée après {self.nodes_explored} nœuds")
        return None


# def create_test_game() -> FreeCellState:
#     """Crée un jeu de test simple (presque résolu)"""
#     state = FreeCellState()
    
#     # Quelques cartes faciles à résoudre
#     state.columns[0] = [Card(10, Suit.HEARTS), Card(9, Suit.HEARTS), Card(13, Suit.DIAMONDS), Card(13, Suit.CLUBS), Card(13, Suit.HEARTS), Card(13, Suit.SPADES)]
#     state.columns[1] = [Card(11, Suit.SPADES), Card(12, Suit.DIAMONDS)]
#     state.columns[2] = [Card(12, Suit.HEARTS), Card(11, Suit.CLUBS)]
#     state.columns[3] = [Card(10, Suit.SPADES), Card(9, Suit.SPADES)]
#     state.columns[4] = [Card(12, Suit.SPADES), Card(10, Suit.DIAMONDS)]
#     state.columns[5] = [Card(10, Suit.CLUBS), Card(11, Suit.DIAMONDS)]
#     state.columns[6] = [Card(12, Suit.CLUBS), Card(9, Suit.DIAMONDS)]
#     state.columns[7] = [Card(11, Suit.HEARTS), Card(9, Suit.CLUBS)]
    
#     # Fondations déjà bien avancées
#     state.foundations = [8, 8, 8, 8]
    
#     return state

# def create_test_game() -> FreeCellState:
#     """Crée un jeu de test simple (presque résolu)"""
#     state = FreeCellState()
    
#     # Quelques cartes faciles à résoudre
#     all_cards = []
#     for suit in [Suit.HEARTS, Suit.DIAMONDS, Suit.CLUBS, Suit.SPADES]:
#         for rank in range(1,14):
#             all_cards.append(Card(rank, suit))
#     random.shuffle(all_cards)

#     state.columns[0] = all_cards[:7]
#     state.columns[1] = all_cards[7:14]
#     state.columns[2] = all_cards[14:21]
#     state.columns[3] = all_cards[21:28]
#     state.columns[4] = all_cards[28:34]
#     state.columns[5] = all_cards[34:40]
#     state.columns[6] = all_cards[40:46]
#     state.columns[7] = all_cards[46:52]

#     # Fondations déjà bien avancées
#     state.foundations = [0,0,0,0]
    
#     return state

def create_test_game() -> FreeCellState:
    """Crée un jeu de test simple (presque résolu)"""
    state = FreeCellState()
    
    k = 4

    # Quelques cartes faciles à résoudre
    all_cards = []
    for suit in [Suit.HEARTS, Suit.DIAMONDS, Suit.CLUBS, Suit.SPADES]:
        for rank in range(k+1,14):
            all_cards.append(Card(rank, suit))
    random.shuffle(all_cards)

    a = len(all_cards) // 8
    b = len(all_cards) % 8

    start = 0
    for i in range(8):
        end = start + a + int(i < b)
        state.columns[i] = all_cards[start:end]
        start, end = end, 0


    # Fondations déjà bien avancées
    state.foundations = [k, k, k, k]
    
    return state


def parse_deal(cards: List[Tuple[int, int]]) -> FreeCellState:
    """Parse un deal depuis un array de 52 tuples (rank, suit)"""
    state = FreeCellState()
    
    # Distribution standard FreeCell: 8 colonnes, 4x7 + 4x6
    col_sizes = [7, 7, 7, 7, 6, 6, 6, 6]
    idx = 0
    
    for col_idx, size in enumerate(col_sizes):
        for _ in range(size):
            rank, suit = cards[idx]
            state.columns[col_idx].append(Card(rank, Suit(suit)))
            idx += 1
    
    return state


if __name__ == "__main__":
    print("=== Solveur FreeCell ===\n")
    
    # Test avec un jeu simple
    state = create_test_game()
    print("État initial:")
    print(state)
    
    solver = FreeCellSolver(state)
    solution = solver.solve(max_nodes=150000)
    
    if solution:
        print("\nSolution:")
        for i, move in enumerate(solution, 1):
            print(f"{i}. {move}")
        
        # Rejouer la solution
        print("\n=== Vérification de la solution ===")
        current = state
        for move in solution:
            current = solver.apply_move(current, move)
        
        print("\nÉtat final:")
        print(current)
        print("Gagné!" if current.is_won() else "Erreur dans la solution")
    
    print("\n--- Pour utiliser avec votre deal de 52 cartes ---")
    print("cards = [(rank, suit), ...] # 52 cartes")
    print("state = parse_deal(cards)")
    print("solver = FreeCellSolver(state)")
    print("solution = solver.solve()")
